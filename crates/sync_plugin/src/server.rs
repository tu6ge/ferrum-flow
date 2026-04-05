use ferrum_flow::{ChangeSource, GraphChange, GraphChangeKind};
use futures::{SinkExt, StreamExt, channel::mpsc::UnboundedSender};
use std::{sync::Arc, thread};
use tokio::runtime::Runtime;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use yrs::{
    Origin, ReadTxn as _, Transact,
    sync::{Awareness, DefaultProtocol, Protocol},
    updates::encoder::{Encode, Encoder, EncoderV1},
};

pub(crate) fn start_sync_thread(
    awareness: Arc<Awareness>,
    undo_origin: Origin,
    repaint_tx: UnboundedSender<GraphChange>,
    ws_url: String,
) {
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async move {
            run_ws(awareness, undo_origin, repaint_tx, ws_url).await;
        });
    });
}

async fn run_ws(
    awareness: Arc<Awareness>,
    undo_origin: Origin,
    repaint_tx: UnboundedSender<GraphChange>,
    ws_url: String,
) {
    let (ws, _) = connect_async(ws_url).await.unwrap();

    let (write, mut read) = ws.split();

    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // === writer ===
    let writer_task = tokio::spawn(async move {
        let mut write = write;
        while let Some(msg) = ws_rx.recv().await {
            match write.send(Message::Binary(msg)).await {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
        }
    });

    // === network receiver ===
    let (incoming_tx, mut incoming_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    let reader_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            let data = match msg {
                Message::Binary(d) => d,
                Message::Close(_) => break,
                _ => continue,
            };

            let _ = incoming_tx.send(data);
        }
    });

    // === GPUI → ws ===
    let ws_tx_clone = ws_tx.clone();
    let _awareness_sub = {
        let awareness = Arc::clone(&awareness);
        let ws_for_awareness = ws_tx.clone();
        let repaint = repaint_tx.clone();
        awareness.on_update(move |aw, ev, _origin| {
            let mine = aw.client_id();
            let local_changed = ev.added().contains(&mine)
                || ev.updated().contains(&mine)
                || ev.removed().contains(&mine);
            if local_changed {
                // `update()` skips clients with no payload; after `clean_local_state` we must send
                // a null entry via `update_with_clients` so peers clear the cursor.
                let au_res = if ev.removed().contains(&mine) {
                    aw.update_with_clients([mine])
                } else {
                    aw.update()
                };
                if let Ok(au) = au_res {
                    let mut enc = EncoderV1::new();
                    yrs::sync::Message::Awareness(au).encode(&mut enc);
                    let _ = ws_for_awareness.send(enc.to_vec());
                }
            }
            let remote_affected = ev.all_changes().iter().any(|&id| id != mine);
            if remote_affected {
                let _ = repaint.unbounded_send(GraphChange {
                    kind: GraphChangeKind::RedrawRequested,
                    source: ChangeSource::Remote,
                });
            }
        })
    };

    let _sub = {
        match awareness.doc().observe_update_v1(move |txn, update| {
            use yrs::sync::{Message, SyncMessage};

            let should_send = matches!(
                txn.origin(),
                Some(o) if *o == yrs::Origin::from("local_intent")
                        || *o == undo_origin
            );

            if !should_send {
                return;
            }

            let msg = Message::Sync(SyncMessage::Update(update.update.clone()));

            let mut encoder = EncoderV1::new();
            msg.encode(&mut encoder);

            ws_tx.send(encoder.to_vec()).unwrap();
        }) {
            Ok(sub) => sub,
            Err(err) => {
                println!("observe_update_v1 failed: {}", err);
                return;
            }
        }
    };

    {
        let sv = awareness.doc().transact().state_vector();
        let step1 = yrs::sync::Message::Sync(yrs::sync::SyncMessage::SyncStep1(sv));
        ws_tx_clone.send(encode_messages([step1])).unwrap();
    }

    if let Ok(au) = awareness.update() {
        let mut enc = EncoderV1::new();
        yrs::sync::Message::Awareness(au).encode(&mut enc);
        ws_tx_clone.send(enc.to_vec()).unwrap();
    }

    // === apply protocol ===
    let applier_task = {
        let awareness = Arc::clone(&awareness);
        tokio::spawn(async move {
            while let Some(data) = incoming_rx.recv().await {
                let before_sv_len = awareness.doc().transact().state_vector().len();

                let replies = match DefaultProtocol.handle(&awareness, &data) {
                    Ok(responses) => responses,
                    Err(_) => {
                        continue;
                    }
                };

                let after_sv_len = awareness.doc().transact().state_vector().len();

                if after_sv_len != before_sv_len {
                    println!(
                        "yrs doc state_vector len changed: {} -> {}",
                        before_sv_len, after_sv_len
                    );
                }

                if !replies.is_empty() {
                    for r in &replies {
                        ws_tx_clone.send(encode_messages([r.clone()])).unwrap();
                    }
                }
            }
        })
    };

    let _ = tokio::join!(writer_task, reader_task, applier_task);
}

fn encode_messages(msgs: impl IntoIterator<Item = yrs::sync::Message>) -> Vec<u8> {
    let mut enc = EncoderV1::new();
    for m in msgs {
        m.encode(&mut enc);
    }
    enc.to_vec()
}
