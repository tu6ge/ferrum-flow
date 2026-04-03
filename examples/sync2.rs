use futures::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use yrs::sync::{Awareness, DefaultProtocol, Protocol, SyncMessage};
use yrs::updates::decoder::Decode as _;
use yrs::updates::encoder::Encoder as _;
use yrs::{
    Doc, ReadTxn as _, Transact,
    updates::encoder::{Encode, EncoderV1},
};

// =====================
// 共享 Hub
// =====================

type ClientHub = Arc<Mutex<Vec<(u64, mpsc::UnboundedSender<Vec<u8>>)>>>;

// =====================
// 判断消息类型
// =====================

enum MsgKind {
    /// 需要广播给其他客户端的 Update
    Update,
    /// 只需点对点回复的握手/Awareness 消息
    PointToPoint,
}

fn classify(data: &[u8]) -> MsgKind {
    use yrs::sync::Message;
    match Message::decode_v1(data) {
        Ok(Message::Sync(SyncMessage::Update(_))) => MsgKind::Update,
        _ => MsgKind::PointToPoint,
    }
}

// =====================
// Server
// =====================

pub async fn run_server() {
    let hub: ClientHub = Arc::new(Mutex::new(Vec::new()));
    let next_id = Arc::new(AtomicU64::new(1));

    // 服务端持有一个权威 Doc，用于：
    //   1. 用 DefaultProtocol 处理握手（SyncStep1 → SyncStep2）
    //   2. 应用所有 Update，保持全量状态
    //   3. 新客户端连接时推送全量快照
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

    // 注册到 hub
    hub.lock().unwrap().push((client_id, out_tx.clone()));

    // 新客户端连上来，立刻推送服务端全量状态（SyncStep1 + SyncStep2）
    // 客户端收到 SyncStep1 后会回一个 SyncStep2 把自己有而服务端没有的部分补上
    {
        let txn = awareness.doc().transact();
        let sv = txn.state_vector();

        // 先发 SyncStep1
        let step1 = yrs::sync::Message::Sync(SyncMessage::SyncStep1(sv));
        let mut enc = EncoderV1::new();
        step1.encode(&mut enc);
        let _ = out_tx.send(enc.to_vec());

        // 再发 SyncStep2（全量）
        let update = txn.encode_state_as_update_v1(&yrs::StateVector::default());
        let step2 = yrs::sync::Message::Sync(SyncMessage::SyncStep2(update));
        let mut enc = EncoderV1::new();
        step2.encode(&mut enc);
        let _ = out_tx.send(enc.to_vec());
    }

    // 读任务：处理来自该客户端的消息
    let hub_read = hub.clone();
    let awareness_read = awareness.clone();
    let out_tx_read = out_tx.clone();

    let read_task = tokio::spawn(async move {
        while let Some(result) = read.next().await {
            let data = match result {
                Ok(Message::Binary(d)) => d,
                Ok(Message::Close(_)) | Err(_) => break,
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

            // 点对点回复（SyncStep1 的回复等）
            for reply in replies {
                let mut enc = EncoderV1::new();
                reply.encode(&mut enc);
                let _ = out_tx_read.send(enc.to_vec());
            }

            // apply 之后，把新增的内容广播给其他所有客户端
            // 用 sv_before 来 diff，只广播本次新增的部分
            let sv_after = awareness_read.doc().transact().state_vector();
            if sv_after != sv_before {
                let diff = awareness_read
                    .doc()
                    .transact()
                    .encode_state_as_update_v1(&sv_before);

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
        }

        println!("[server] client {} disconnected", client_id);
        hub_read.lock().unwrap().retain(|(id, _)| *id != client_id);
    });

    // 写任务：把所有出站消息发给该客户端
    let write_task = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if write.send(Message::Binary(msg)).await.is_err() {
                break;
            }
        }
    });

    // 任意一个任务退出就取消另一个，避免泄漏
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
