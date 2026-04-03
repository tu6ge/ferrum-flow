use futures::{SinkExt, StreamExt};
use std::{sync::Arc, thread};
use tokio::runtime::Runtime;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use yrs::{
    Doc, ReadTxn as _, Transact,
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
    let awareness = Arc::new(Awareness::new(doc));

    let _ = awareness.set_local_state(r#"{"name":"Alice2"}"#);

    let (ws, _) = connect_async("ws://127.0.0.1:9001").await.unwrap();

    let (write, mut read) = ws.split();

    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // === writer ===
    let writer_task = tokio::spawn(async move {
        let mut write = write;

        while let Some(msg) = ws_rx.recv().await {
            let len = msg.len();
            match write.send(Message::Binary(msg)).await {
                Ok(_) => {
                    println!(">>> WS SEND: {} bytes", len);
                }
                Err(e) => {
                    println!("WS SEND failed: {}", e);
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

            println!("<<< WS RECV: {} bytes", data.len());
            let _ = incoming_tx.send(data);
        }
    });

    // === GPUI → ws ===
    let ws_tx_clone = ws_tx.clone();
    let _sub = {
        match awareness.doc().observe_update_v1(move |txn, update| {
            use yrs::sync::{Message, SyncMessage};

            // 非 local_intent 来源的 update 不转发（远端同步进来的）
            if !matches!(txn.origin(), Some(o) if *o == yrs::Origin::from("local_intent")) {
                return;
            }

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
        // let applying_remote_clone = Arc::clone(&applying_remote);
        let awareness = Arc::clone(&awareness);
        tokio::spawn(async move {
            while let Some(data) = incoming_rx.recv().await {
                let before_sv_len = awareness.doc().transact().state_vector().len();

                //applying_remote_clone.store(true, Ordering::SeqCst);
                let replies = match DefaultProtocol.handle(&awareness, &data) {
                    Ok(responses) => responses,
                    Err(e) => {
                        //applying_remote_clone.store(false, Ordering::SeqCst);
                        println!("protocol error: {:?}", e);
                        continue;
                    }
                };
                //applying_remote_clone.store(false, Ordering::SeqCst);

                let after_sv_len = awareness.doc().transact().state_vector().len();
                if after_sv_len != before_sv_len {
                    println!(
                        "yrs doc state_vector len changed: {} -> {}",
                        before_sv_len, after_sv_len
                    );
                }

                if !replies.is_empty() {
                    println!("protocol replies: {}", replies.len());
                    for r in &replies {
                        match ws_tx_clone.send(encode_messages([r.clone()])) {
                            Ok(_) => {
                                println!("response sent");
                            }
                            Err(e) => {
                                println!("send failed2: {}", e);
                                break;
                            }
                        }
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
