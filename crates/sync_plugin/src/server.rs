use ferrum_flow::{ChangeSource, GraphChange, GraphChangeKind};
use futures::{SinkExt, StreamExt, channel::mpsc::UnboundedSender};
use std::{sync::Arc, thread, time::Duration};
use tokio::runtime::Runtime;
use tokio::time::sleep;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use yrs::{
    Origin, ReadTxn as _, Transact,
    sync::{Awareness, DefaultProtocol, Protocol},
    updates::encoder::{Encode, Encoder, EncoderV1},
};

/// WebSocket sync client behaviour: reconnect after disconnect and bounded retries per connect phase.
#[derive(Clone, Debug)]
pub struct WsSyncConfig {
    /// Maximum connection attempts per phase (initial connect or after a dropped session).
    /// Set to `u32::MAX` to retry until success.
    pub max_connect_retries: u32,
    /// Delay after the first failed `connect_async`, before the second attempt.
    pub retry_initial_backoff: Duration,
    /// Upper bound for exponential backoff between failed connects.
    pub retry_max_backoff: Duration,
    /// Pause after the remote closes or the read loop ends, before trying to connect again.
    pub reconnect_delay: Duration,
}

impl Default for WsSyncConfig {
    fn default() -> Self {
        Self {
            max_connect_retries: 10,
            retry_initial_backoff: Duration::from_secs(1),
            retry_max_backoff: Duration::from_secs(30),
            reconnect_delay: Duration::from_secs(1),
        }
    }
}

pub(crate) fn start_sync_thread(
    awareness: Arc<Awareness>,
    undo_origin: Origin,
    repaint_tx: UnboundedSender<GraphChange>,
    ws_url: String,
    ws_config: WsSyncConfig,
) {
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async move {
            run_ws(awareness, undo_origin, repaint_tx, ws_url, ws_config).await;
        });
    });
}

async fn run_ws(
    awareness: Arc<Awareness>,
    undo_origin: Origin,
    repaint_tx: UnboundedSender<GraphChange>,
    ws_url: String,
    config: WsSyncConfig,
) {
    loop {
        let ws = match connect_with_retries(&ws_url, &config).await {
            Some(s) => s,
            None => {
                if config.max_connect_retries == u32::MAX {
                    eprintln!("[ferrum-flow-sync] connect loop exited unexpectedly");
                } else {
                    eprintln!(
                        "[ferrum-flow-sync] giving up after {} failed connect attempt(s)",
                        config.max_connect_retries
                    );
                }
                return;
            }
        };

        run_one_session(ws, Arc::clone(&awareness), undo_origin.clone(), repaint_tx.clone()).await;

        sleep(config.reconnect_delay).await;
    }
}

/// Returns `None` if all attempts in this phase failed.
async fn connect_with_retries(
    url: &str,
    config: &WsSyncConfig,
) -> Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
    let mut delay = config.retry_initial_backoff;
    let mut attempt: u32 = 0;
    let unlimited = config.max_connect_retries == u32::MAX;

    loop {
        attempt = attempt.saturating_add(1);
        match connect_async(url).await {
            Ok((ws, _)) => return Some(ws),
            Err(e) => {
                eprintln!(
                    "[ferrum-flow-sync] WebSocket connect failed (attempt {}): {}",
                    attempt, e
                );
                if !unlimited && attempt >= config.max_connect_retries {
                    return None;
                }
                sleep(delay).await;
                delay = delay
                    .checked_mul(2)
                    .unwrap_or(config.retry_max_backoff)
                    .min(config.retry_max_backoff);
            }
        }
    }
}

async fn run_one_session(
    ws: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    awareness: Arc<Awareness>,
    undo_origin: Origin,
    repaint_tx: UnboundedSender<GraphChange>,
) {
    let (mut write, mut read) = ws.split();

    let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let (incoming_tx, mut incoming_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    let writer_task = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if write.send(Message::Binary(msg)).await.is_err() {
                break;
            }
        }
    });

    let incoming_for_reader = incoming_tx.clone();
    let reader_task = tokio::spawn(async move {
        while let Some(result) = read.next().await {
            let data = match result {
                Ok(Message::Binary(d)) => d,
                Ok(Message::Close(_)) | Err(_) => break,
                _ => continue,
            };
            let _ = incoming_for_reader.send(data);
        }
    });

    let out_for_applier = out_tx.clone();
    let awareness_applier = Arc::clone(&awareness);
    let applier_task = tokio::spawn(async move {
        while let Some(data) = incoming_rx.recv().await {
            let before_sv_len = awareness_applier.doc().transact().state_vector().len();

            let replies = match DefaultProtocol.handle(&awareness_applier, &data) {
                Ok(responses) => responses,
                Err(_) => continue,
            };

            let after_sv_len = awareness_applier.doc().transact().state_vector().len();

            if after_sv_len != before_sv_len {
                eprintln!(
                    "[ferrum-flow-sync] yrs doc state_vector len changed: {} -> {}",
                    before_sv_len, after_sv_len
                );
            }

            if !replies.is_empty() {
                for r in &replies {
                    let _ = out_for_applier.send(encode_messages([r.clone()]));
                }
            }
        }
    });

    let out_awareness = out_tx.clone();
    let repaint_aw = repaint_tx.clone();
    let _awareness_sub = {
        let awareness = Arc::clone(&awareness);
        awareness.on_update(move |aw, ev, _origin| {
            let mine = aw.client_id();
            let local_changed = ev.added().contains(&mine)
                || ev.updated().contains(&mine)
                || ev.removed().contains(&mine);
            if local_changed {
                let au_res = if ev.removed().contains(&mine) {
                    aw.update_with_clients([mine])
                } else {
                    aw.update()
                };
                if let Ok(au) = au_res {
                    let mut enc = EncoderV1::new();
                    yrs::sync::Message::Awareness(au).encode(&mut enc);
                    let _ = out_awareness.send(enc.to_vec());
                }
            }
            let remote_affected = ev.all_changes().iter().any(|&id| id != mine);
            if remote_affected {
                let _ = repaint_aw.unbounded_send(GraphChange {
                    kind: GraphChangeKind::RedrawRequested,
                    source: ChangeSource::Remote,
                });
            }
        })
    };

    let out_doc = out_tx.clone();
    let _doc_sub = match awareness.doc().observe_update_v1(move |txn, update| {
        use yrs::sync::{Message, SyncMessage};

        let should_send = matches!(
            txn.origin(),
            Some(o) if *o == yrs::Origin::from("local_intent") || *o == undo_origin
        );

        if !should_send {
            return;
        }

        let msg = Message::Sync(SyncMessage::Update(update.update.clone()));

        let mut encoder = EncoderV1::new();
        msg.encode(&mut encoder);

        let _ = out_doc.send(encoder.to_vec());
    }) {
        Ok(sub) => sub,
        Err(err) => {
            eprintln!("[ferrum-flow-sync] observe_update_v1 failed: {}", err);
            drop(incoming_tx);
            reader_task.abort();
            applier_task.abort();
            writer_task.abort();
            let _ = reader_task.await;
            let _ = applier_task.await;
            let _ = writer_task.await;
            return;
        }
    };

    {
        let sv = awareness.doc().transact().state_vector();
        let step1 = yrs::sync::Message::Sync(yrs::sync::SyncMessage::SyncStep1(sv));
        let _ = out_tx.send(encode_messages([step1]));
    }

    if let Ok(au) = awareness.update() {
        let mut enc = EncoderV1::new();
        yrs::sync::Message::Awareness(au).encode(&mut enc);
        let _ = out_tx.send(enc.to_vec());
    }

    drop(out_tx);

    let _ = reader_task.await;
    drop(incoming_tx);
    let _ = applier_task.await;

    drop(_doc_sub);
    drop(_awareness_sub);

    let _ = writer_task.await;
}

fn encode_messages(msgs: impl IntoIterator<Item = yrs::sync::Message>) -> Vec<u8> {
    let mut enc = EncoderV1::new();
    for m in msgs {
        m.encode(&mut enc);
    }
    enc.to_vec()
}
