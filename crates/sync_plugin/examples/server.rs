use futures::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use yrs::encoding::read::Cursor;
use yrs::sync::{Awareness, DefaultProtocol, MessageReader, Protocol, SyncMessage};
use yrs::updates::decoder::{Decode as _, DecoderV1};
use yrs::updates::encoder::Encoder as _;
use yrs::{
    Doc, ReadTxn as _, Transact, Update,
    updates::encoder::{Encode, EncoderV1},
};

// =====================
// share Hub
// =====================

type ClientHub = Arc<Mutex<Vec<(u64, mpsc::UnboundedSender<Vec<u8>>)>>>;

// =====================
// Server
// =====================

pub async fn run_server() {
    let hub: ClientHub = Arc::new(Mutex::new(Vec::new()));
    let next_id = Arc::new(AtomicU64::new(1));

    // the server has an authoritative Doc, used for:
    //   1. use DefaultProtocol to handle the handshake (SyncStep1 → SyncStep2)
    //   2. apply all Updates, keep the full state
    //   3. push the full snapshot when a new client connects
    let server_doc = Arc::new(Doc::new());
    let awareness = Arc::new(Awareness::new((*server_doc).clone()));

    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    println!("[server] listening on 127.0.0.1:9001");

    while let Ok((stream, addr)) = listener.accept().await {
        let hub = hub.clone();
        let awareness = awareness.clone();
        let client_id = next_id.fetch_add(1, Ordering::Relaxed);

        println!("[server] client {} connected: {}", client_id, addr);

        tokio::spawn(handle_client(stream, client_id, hub, awareness));
    }
}

async fn handle_client(
    stream: tokio::net::TcpStream,
    client_id: u64,
    hub: ClientHub,
    awareness: Arc<Awareness>,
) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("[server] ws handshake failed for {}: {}", client_id, e);
            return;
        }
    };

    let (mut write, mut read) = ws.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // register to hub
    hub.lock().unwrap().push((client_id, out_tx.clone()));

    // a new client connects, immediately push the full state of the server (SyncStep1 + SyncStep2)
    // the client receives SyncStep1 and returns SyncStep2 to fill in the parts that the server does not have
    {
        let txn = awareness.doc().transact();
        let sv = txn.state_vector();

        // send SyncStep1 first
        let step1 = yrs::sync::Message::Sync(SyncMessage::SyncStep1(sv));
        let mut enc = EncoderV1::new();
        step1.encode(&mut enc);
        let _ = out_tx.send(enc.to_vec());

        // send SyncStep2 (full)
        let update = txn.encode_state_as_update_v1(&yrs::StateVector::default());
        let step2 = yrs::sync::Message::Sync(SyncMessage::SyncStep2(update));
        let mut enc = EncoderV1::new();
        step2.encode(&mut enc);
        let _ = out_tx.send(enc.to_vec());

        // Merge everyone else's awareness (cursors, etc.) into this new client.
        if let Ok(au) = awareness.update() {
            let mut enc = EncoderV1::new();
            yrs::sync::Message::Awareness(au).encode(&mut enc);
            let _ = out_tx.send(enc.to_vec());
        }
    }

    // read task: handle messages from the client
    let hub_read = hub.clone();
    let awareness_read = awareness.clone();
    let out_tx_read = out_tx.clone();

    let read_task = tokio::spawn(async move {
        while let Some(result) = read.next().await {
            let data = match result {
                Ok(WsMessage::Binary(d)) => d,
                Ok(WsMessage::Close(_)) | Err(_) => break,
                _ => continue,
            };

            let sv_before = awareness_read.doc().transact().state_vector();

            let replies = match DefaultProtocol.handle(&awareness_read, &data) {
                Ok(r) => r,
                Err(e) => {
                    println!("[server] protocol error from {}: {:?}", client_id, e);
                    continue;
                }
            };

            // point-to-point replies (replies for SyncStep1)
            for reply in replies {
                let mut enc = EncoderV1::new();
                reply.encode(&mut enc);
                let _ = out_tx_read.send(enc.to_vec());
            }

            // Decide broadcast using the delta relative to `sv_before`; do not rely only on whether the
            // state vector changed: pure deletes often leave every client's clock unchanged, yet the
            // encoded Update still carries a delete set and must be forwarded.
            let diff = awareness_read
                .doc()
                .transact()
                .encode_state_as_update_v1(&sv_before);

            let should_broadcast = Update::decode_v1(diff.as_slice())
                .map(|u| !u.is_empty())
                .unwrap_or(!diff.is_empty());

            if should_broadcast {
                let broadcast_msg = yrs::sync::Message::Sync(yrs::sync::SyncMessage::Update(diff));
                let mut enc = EncoderV1::new();
                broadcast_msg.encode(&mut enc);
                let bytes = enc.to_vec();

                let targets: Vec<_> = hub_read
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|(id, _)| *id != client_id)
                    .map(|(_, tx)| tx.clone())
                    .collect();

                for tx in targets {
                    let _ = tx.send(bytes.clone());
                }
            }

            // Awareness is separate from the Y.Doc: relay each Awareness frame so other peers get cursors.
            let mut dec = DecoderV1::new(Cursor::new(data.as_slice()));
            let mut reader = MessageReader::new(&mut dec);
            while let Some(msg_res) = reader.next() {
                let Ok(msg) = msg_res else {
                    break;
                };
                let yrs::sync::Message::Awareness(au) = msg else {
                    continue;
                };
                let mut enc = EncoderV1::new();
                yrs::sync::Message::Awareness(au).encode(&mut enc);
                let bytes = enc.to_vec();
                let targets: Vec<_> = hub_read
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|(id, _)| *id != client_id)
                    .map(|(_, tx)| tx.clone())
                    .collect();
                for tx in targets {
                    let _ = tx.send(bytes.clone());
                }
            }
        }

        println!("[server] client {} disconnected", client_id);
        hub_read.lock().unwrap().retain(|(id, _)| *id != client_id);
    });

    // write task: send all outgoing messages to the client
    let write_task = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if write.send(WsMessage::Binary(msg)).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = read_task => {}
        _ = write_task => {}
    }
}

// =====================
// main
// =====================

#[tokio::main]
async fn main() {
    run_server().await;
}
