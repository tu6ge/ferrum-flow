use futures::{SinkExt, StreamExt};
use std::thread;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use yrs::{
    Doc, ReadTxn as _, StateVector, Transact,
    encoding::{read::Read as _, write::Write as _},
    updates::{
        decoder::{Decode as _, DecoderV1},
        encoder::{Encode, Encoder, EncoderV1},
    },
};

pub(super) fn start_sync_thread(doc: yrs::Doc) -> UnboundedSender<Vec<u8>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async move {
            run_ws(doc, rx).await;
        });
    });

    tx
}
async fn run_ws(doc: Doc, mut rx: UnboundedReceiver<Vec<u8>>) {
    let (ws, _) = connect_async("ws://localhost:1234/my-roomname")
        .await
        .unwrap();

    let (write, mut read) = ws.split();

    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // === writer task (only write)===
    let writer_task = tokio::spawn(async move {
        let mut write = write;

        while let Some(msg) = ws_rx.recv().await {
            match write.send(msg.into()).await {
                Ok(_) => println!("send to ws success"),
                Err(e) => println!("send to ws faild: {}", e),
            }
        }
    });

    // === init sync ===
    {
        let txn = doc.transact();
        let sv = txn.state_vector().encode_v1();

        let mut msg = vec![0];
        msg.extend(sv);

        ws_tx.send(msg).unwrap();
    }

    // === reader task ===
    let doc_clone = doc.clone();
    let ws_tx_clone = ws_tx.clone();

    let reader_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            let msg = match msg {
                Message::Binary(d) => d,
                Message::Close(_) => break,
                _ => continue, // ignore Ping, Pong, Text
            };

            if msg.len() < 1 {
                continue;
            }

            //if msg[0] == 0 {
            match msg[0] {
                0 => {
                    if let Ok(sv) = StateVector::decode_v1(&msg[1..]) {
                        let update = {
                            let txn = doc_clone.transact();
                            txn.encode_state_as_update_v1(&sv)
                        };

                        let mut reply = vec![1];
                        reply.extend(update);

                        let _ = ws_tx_clone.send(reply);
                        println!("send to ws");
                    } else {
                        println!("decode failed");
                    }
                }

                1 | 2 => {
                    println!("ssssss 1|2");
                    if let Ok(update) = yrs::Update::decode_v1(&msg[1..]) {
                        let mut txn = doc_clone.transact_mut();
                        if txn.apply_update(update).is_ok() {
                            println!("Applied remote update from WS");
                        } else {
                            println!("apply update failed");
                        }
                    } else {
                        println!("error 222");
                    }
                }

                _ => {}
            }
            // } else if msg[0] == 1 {
            //     // handler Awareness
            // }
        }
    });

    // === GPUI msg to websocket ===
    let rx_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match ws_tx.send(msg) {
                Ok(_) => println!("GPUI msg to websocket success send"),
                Err(e) => println!("GPUI msg to websocket send error: {}", e),
            }
        }
    });

    tokio::select! {
        _ = writer_task => {},
        _ = reader_task => {},
    };
    rx_task.abort();
}
