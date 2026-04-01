use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use yrs::MapRef;
use yrs::Out;
use yrs::Subscription;
use yrs::updates::decoder::Decode as _;
use yrs::{Doc, Map, ReadTxn as _, StateVector, Transact, Update};

// =====================
// Graph CRDT 封装
// =====================

#[derive(Clone)]
struct GraphCRDT {
    doc: Arc<Doc>,
    applying_remote: Arc<AtomicBool>,
}

impl GraphCRDT {
    fn new() -> Self {
        let doc = Doc::new();

        let graph = doc.get_or_insert_map("graph");
        let mut txn = doc.transact_mut();
        let _nodes: MapRef = graph.get_or_init(&mut txn, "nodes");
        drop(txn);

        Self {
            doc: Arc::new(doc),
            applying_remote: Arc::new(AtomicBool::new(false)),
        }
    }

    fn add_node(&self, id: u64, x: f64, y: f64) {
        let graph = self.doc.get_or_insert_map("graph");

        let mut txn = self.doc.transact_mut();
        let nodes: MapRef = graph.get_or_init(&mut txn, "nodes");
        let node: MapRef = nodes.get_or_init(&mut txn, id.to_string());
        node.insert(&mut txn, "x", x);
        node.insert(&mut txn, "y", y);
    }

    fn move_node(&self, id: u64, x: f64, y: f64) {
        let graph = self.doc.get_or_insert_map("graph");

        let mut txn = self.doc.transact_mut();
        let nodes: MapRef = graph.get_or_init(&mut txn, "nodes");

        if let Some(Out::YMap(node)) = nodes.get(&txn, &id.to_string()) {
            node.try_update(&mut txn, "x", x);
            node.try_update(&mut txn, "y", y);
        }
    }

    fn apply_update(&self, bytes: &[u8]) {
        let update = Update::decode_v1(bytes).unwrap();
        self.applying_remote.store(true, Ordering::SeqCst);
        let mut txn = self.doc.transact_mut();
        txn.apply_update(update).unwrap();
        self.applying_remote.store(false, Ordering::SeqCst);
    }

    fn encode_update(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    fn observe<F>(&self, f: F) -> Subscription
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let doc = self.doc.clone();
        let applying_remote = self.applying_remote.clone();

        doc.observe_update_v1(move |_, update| {
            if applying_remote.load(Ordering::SeqCst) {
                return;
            }
            f(update.update.to_vec());
        })
        .unwrap()
    }

    fn print(&self) {
        let graph = self.doc.get_or_insert_map("graph");
        let txn = self.doc.transact();

        let Some(Out::YMap(nodes)) = graph.get(&txn, "nodes") else {
            return;
        };

        println!("--- Graph State ---");
        let mut has_nodes = false;
        for (k, v) in nodes.iter(&txn) {
            if let Out::YMap(node) = v {
                has_nodes = true;
                let x = node.get(&txn, "x").unwrap();
                let y = node.get(&txn, "y").unwrap();

                println!("Node {} => x: {}, y: {}", k, x, y);
            }
        }

        if !has_nodes {
            println!("No nodes");
        }
    }
}

// =====================
// WebSocket Server
// =====================

async fn run_server(tx: broadcast::Sender<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let ws = accept_async(stream).await.unwrap();
            let (mut write, mut read) = ws.split();

            // 收消息 → 广播
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                while let Some(Ok(msg)) = read.next().await {
                    if let Message::Binary(data) = msg {
                        let _ = tx_clone.send(data);
                    }
                }
            });

            // 广播 → 发给客户端
            while let Ok(msg) = rx.recv().await {
                let _ = write.send(Message::Binary(msg)).await;
            }
        });
    }
}

// =====================
// WebSocket Client
// =====================

async fn run_client(name: &'static str) {
    // 处理 server 尚未就绪的竞态，避免 spawn 任务里 panic 后静默退出
    let ws = loop {
        match connect_async("ws://127.0.0.1:9001").await {
            Ok((ws, _)) => break ws,
            Err(err) => {
                println!("[{}] connect failed: {}. retrying...", name, err);
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }
        }
    };

    let (write, mut read) = ws.split();

    let graph = GraphCRDT::new();

    // 在 observe 回调里避免阻塞：只入队，由独立异步任务发送 websocket 消息
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    tokio::spawn(async move {
        let mut write = write;
        while let Some(update) = out_rx.recv().await {
            if write.send(Message::Binary(update)).await.is_err() {
                break;
            }
        }
    });
    let out_tx_for_observe = out_tx.clone();
    let _sub = graph.observe(move |update| {
        let _ = out_tx_for_observe.send(update);
    });

    // 首次连接主动发一次当前全量状态，避免错过早期增量
    let init_update = graph.encode_update();
    let _ = out_tx.send(init_update);

    // 接收远程更新
    let graph_clone = graph.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Binary(data) = msg {
                graph_clone.apply_update(&data);
                //println!("[{}] received update", name);
                //graph_clone.print();
            }
        }
    });

    // 模拟操作：先由 A 创建节点
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    if name == "A" {
        graph.add_node(1, 0.0, 0.0);
        println!("[A] add node (0, 0)");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        graph.print();
    }

    // 第 1 轮：B 拖动，观察 A 是否收到
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    if name == "B" {
        graph.move_node(1, 100.0, 200.0);
        println!("[B] move node -> (100, 200)");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        graph.print();
    }

    // 第 2 轮：A 再拖动，观察 B 是否收到
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    if name == "A" {
        println!("[A] print");
        graph.print();
        graph.move_node(1, 300.0, 400.0);
        println!("[A] move node -> (300, 400)");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        graph.print();
    }

    // 第 2 轮：A 再拖动，观察 B 是否收到
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;

    if name == "B" {
        println!("[B] print");
        graph.print();
    }

    loop {
        // 显式持有订阅，避免被编译器提前 drop
        //let _keep_observer_alive = &sub;
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

// =====================
// main
// =====================

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel(100);

    // 启动服务器
    tokio::spawn(run_server(tx.clone()));
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // 启动两个客户端
    tokio::spawn(run_client("A"));
    tokio::spawn(run_client("B"));

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
