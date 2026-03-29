use futures::{SinkExt, StreamExt};
use std::{sync::Arc, thread};
use tokio::runtime::Runtime;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use yrs::{
    Doc,
    sync::{Awareness, DefaultProtocol, Protocol},
    updates::encoder::{Encode, Encoder, EncoderV1},
};

pub(super) fn start_sync_thread(doc: yrs::Doc) {
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async move {
            run_ws(doc).await;
        });
    });
}

async fn run_ws(doc: Doc) {
    let awareness = Arc::new(Awareness::new(doc.clone()));
    let protocol = DefaultProtocol;

    let (ws, _) = connect_async("ws://localhost:1234/my-room").await.unwrap();

    let (write, mut read) = ws.split();

    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // === writer ===
    let writer_task = tokio::spawn(async move {
        let mut write = write;

        while let Some(msg) = ws_rx.recv().await {
            if let Err(e) = write.send(msg.into()).await {
                println!("send failed: {}", e);
                break;
            }
        }
    });

    // === init sync ===
    {
        let mut encoder = EncoderV1::new();
        protocol.start(&awareness, &mut encoder).unwrap();
        match ws_tx.send(encoder.to_vec()) {
            Ok(_) => {
                println!("initial sync sent");
            }
            Err(e) => {
                println!("send failed: {}", e);
            }
        }
    }

    // === reader ===
    let ws_tx_clone = ws_tx.clone();
    let awareness_clone = awareness.clone();

    let reader_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            let data = match msg {
                Message::Binary(d) => d,
                Message::Close(_) => break,
                _ => continue,
            };

            match protocol.handle(&awareness, &data) {
                Ok(responses) => {
                    for msg in responses {
                        let mut encoder = EncoderV1::new();
                        msg.encode(&mut encoder);

                        match ws_tx_clone.send(encoder.to_vec()) {
                            Ok(_) => {
                                println!("response sent");
                            }
                            Err(e) => {
                                println!("send failed: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("protocol error: {:?}", e);
                }
            }
        }
    });

    // === GPUI → ws ===
    let ws_tx = ws_tx.clone();

    let _sub = awareness_clone
        .doc()
        .observe_update_v1(move |_, update| {
            use yrs::sync::{Message, SyncMessage};

            let msg = Message::Sync(SyncMessage::Update(update.update.clone()));

            let mut encoder = EncoderV1::new();
            msg.encode(&mut encoder);

            match ws_tx.send(encoder.to_vec()) {
                Ok(_) => {
                    println!("sync message sent");
                }
                Err(e) => {
                    println!("sended sync message failed: {}", e);
                }
            }
        })
        .unwrap();

    tokio::select! {
        _ = writer_task => {},
        _ = reader_task => {},
    };
}
