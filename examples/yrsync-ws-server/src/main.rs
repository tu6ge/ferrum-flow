//! Minimal WebSocket relay for Yrs / y-sync binary frames.
//!
//! Run from repo root (workspace member, shares `Cargo.lock` with `ferrum-flow`):
//!
//! ```text
//! cargo run -p yrsync-ws-server
//! cargo run -p yrsync-ws-server -- 0.0.0.0:8080
//! ```
//!
//! Clients connect to `ws://127.0.0.1:8080` (default). Each incoming **binary**
//! message is forwarded to every other connected peer in the same room.
//!
//! Rooms are selected by URL path, e.g. `ws://127.0.0.1:8080/default` — peers
//! only see traffic from others in the same path. `/` is normalized to `/default`.

use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{net::TcpListener, net::TcpStream, sync::RwLock, sync::mpsc};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        Message,
        handshake::server::{Request, Response},
    },
};

static NEXT_PEER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
struct Peer {
    id: u64,
    tx: mpsc::UnboundedSender<Vec<u8>>,
}

type RoomPeers = Arc<RwLock<Vec<Peer>>>;

#[derive(Clone, Default)]
struct Rooms {
    inner: Arc<RwLock<HashMap<String, RoomPeers>>>,
}

impl Rooms {
    async fn room(&self, name: String) -> RoomPeers {
        let mut map = self.inner.write().await;
        map.entry(name)
            .or_insert_with(|| Arc::new(RwLock::new(Vec::new())))
            .clone()
    }
}

fn normalize_room(path: &str) -> String {
    let p = path.trim();
    if p.is_empty() || p == "/" {
        "default".to_owned()
    } else {
        p.trim_start_matches('/').to_owned()
    }
}

/// Handshake: capture path for room, then run read/write loop.
async fn run_peer(
    ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    rooms: Rooms,
    room_name: String,
) -> Result<()> {
    let room = rooms.room(room_name.clone()).await;
    let peer_id = NEXT_PEER_ID.fetch_add(1, Ordering::Relaxed);
    let (peer_tx, mut peer_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    {
        let mut peers = room.write().await;
        peers.push(Peer {
            id: peer_id,
            tx: peer_tx,
        });
        println!(
            "peer {} joined room '{}' ({} peers)",
            peer_id,
            room_name,
            peers.len()
        );
    }

    let (mut write, mut read) = ws_stream.split();

    let res = loop {
        tokio::select! {
            out = peer_rx.recv() => {
                let Some(data) = out else { break Ok(()); };
                if write.send(Message::Binary(data)).await.is_err() {
                    break Err(anyhow::anyhow!("send failed"));
                }
            }
            incoming = read.next() => {
                let Some(incoming) = incoming else { break Ok(()); };
                let msg = incoming.context("websocket read")?;
                match msg {
                    Message::Binary(data) => {
                        let peers = room.read().await;
                        for p in peers.iter() {
                            if p.id != peer_id && p.tx.send(data.clone()).is_err() {
                                // peer queue closed; ignore
                            }
                        }
                    }
                    Message::Ping(payload) => {
                        if write.send(Message::Pong(payload)).await.is_err() {
                            break Err(anyhow::anyhow!("pong failed"));
                        }
                    }
                    Message::Close(_) => break Ok(()),
                    Message::Text(_) | Message::Pong(_) | Message::Frame(_) => {}
                }
            }
        }
    };

    {
        let mut peers = room.write().await;
        peers.retain(|p| p.id != peer_id);
        println!(
            "peer {} left room '{}' ({} peers)",
            peer_id,
            room_name,
            peers.len()
        );
    }

    res
}

#[tokio::main]
async fn main() -> Result<()> {
    let listen = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let listener = TcpListener::bind(&listen)
        .await
        .with_context(|| format!("bind {}", listen))?;

    println!(
        "yrsync-ws-server: relay on ws://{}/<room>  (default room if path is /)",
        listen
    );

    let rooms = Rooms::default();

    while let Ok((stream, peer_addr)) = listener.accept().await {
        let rooms = rooms.clone();
        tokio::spawn(async move {
            let mut room_name = String::from("default");
            let ws = match accept_hdr_async(stream, |req: &Request, response: Response| {
                room_name = normalize_room(req.uri().path());
                Ok(response)
            })
            .await
            {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("accept {}: {}", peer_addr, e);
                    return;
                }
            };

            if let Err(e) = run_peer(ws, rooms, room_name).await {
                eprintln!("peer {} session: {}", peer_addr, e);
            }
        });
    }

    Ok(())
}
