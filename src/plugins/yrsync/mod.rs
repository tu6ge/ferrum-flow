use std::{sync::Arc, vec};

use gpui::{Size, px};
use serde_json::Value;
use tokio::{runtime::Runtime, sync::mpsc::Sender};
use yrs::{
    Any, Array as _, ArrayRef, DeepObservable, Doc, Map, MapPrelim, MapRef, Observable as _,
    Origin, Out, Transact, TransactionMut,
    types::{DefaultPrelim, EntryChange, PathSegment},
    undo::Options,
};

use crate::{
    ChangeSource, Edge, EdgeId, Graph, GraphChange, GraphChangeKind, GraphOp, Node, NodeId, Port,
    PortId, SyncPlugin,
};

pub struct YrsSyncPlugin {
    doc: yrs::Doc,
    init_graph: Graph,
    pub nodes: MapRef,        // ref HashMap<NodeId, Node>
    pub ports: MapRef,        // ref HashMap<PortId, Port>
    pub edges: MapRef,        // ref HashMap<EdgeId, Edge>
    pub node_order: ArrayRef, // ref Vec<NodeId>
    undo_manager: yrs::UndoManager,
    _subscription_nodes: Option<yrs::Subscription>,
    _subscription_ports: Option<yrs::Subscription>,
    _subscription_edges: Option<yrs::Subscription>,
    _subscription_order: Option<yrs::Subscription>,
}

impl YrsSyncPlugin {
    pub fn new(graph: Graph) -> Self {
        let doc = Doc::new();
        let root = doc.get_or_insert_map("graph");
        let mut txn = doc.transact_mut();
        let nodes = root.get_or_init(&mut txn, "nodes");
        let ports = root.get_or_init(&mut txn, "ports");
        let edges = root.get_or_init(&mut txn, "edges");
        let node_order = root.get_or_init(&mut txn, "node_order");

        drop(txn);

        let mut option = Options::default();
        option.tracked_origins.insert("local_intent".into());
        let mut undo_manager = yrs::UndoManager::with_scope_and_options(&doc, &root, option);
        undo_manager.include_origin(doc.client_id());

        Self {
            init_graph: graph,
            undo_manager,
            doc,
            nodes,
            ports,
            edges,
            node_order,
            _subscription_nodes: None,
            _subscription_ports: None,
            _subscription_edges: None,
            _subscription_order: None,
        }
    }

    pub fn from_graph(&self) {
        let mut txn: TransactionMut<'_> = self.doc.transact_mut_with("local_init");
        for node in self.init_graph.nodes().values() {
            self.insert_node(&mut txn, node);
        }
        for port in self.init_graph.ports.values() {
            self.add_port(&mut txn, port);
        }
        for edge in self.init_graph.edges.values() {
            self.insert_edge(&mut txn, edge);
        }
        for order in self.init_graph.node_order() {
            self.add_node_order(&mut txn, order);
        }
    }

    fn insert_node(&self, txn: &mut TransactionMut, node: &Node) {
        let node_map = MapPrelim::default();
        let node_ref = self.nodes.insert(txn, node.id.0.to_string(), node_map);

        node_ref.insert(txn, "type", node.node_type.clone());
        node_ref.insert(txn, "x", Into::<f32>::into(node.x));
        node_ref.insert(txn, "y", Into::<f32>::into(node.y));
        node_ref.insert(txn, "width", Into::<f32>::into(node.size.width));
        node_ref.insert(txn, "height", Into::<f32>::into(node.size.height));

        let inputs = node_ref.insert(txn, "inputs", ArrayRef::default_prelim());
        for port_id in &node.inputs {
            inputs.push_front(txn, port_id.0.to_string());
        }
        let outputs = node_ref.insert(txn, "outputs", ArrayRef::default_prelim());
        for port_id in &node.outputs {
            outputs.push_front(txn, port_id.0.to_string());
        }

        let data_json = serde_json::to_string(&node.data).unwrap_or_default();
        node_ref.insert(txn, "data", data_json);
    }

    fn update_node_position(&self, txn: &mut TransactionMut, id: &NodeId, x: f32, y: f32) {
        if let Some(yrs::Out::YMap(node_ref)) = self.nodes.get(txn, &id.0.to_string()) {
            node_ref.try_update(txn, "x", x);
            node_ref.try_update(txn, "y", y);
        }
    }

    fn add_node_order(&self, txn: &mut TransactionMut, id: &NodeId) {
        self.node_order.push_back(txn, id.0.to_string());
    }

    fn remove_noder_order(&self, txn: &mut TransactionMut, index: usize) {
        self.node_order.remove(txn, index as u32);
    }

    fn remove_node(&self, txn: &mut TransactionMut, id: &NodeId) {
        self.nodes.remove(txn, &id.0.to_string());
    }

    fn add_port(&self, txn: &mut TransactionMut, port: &Port) {
        let port_map = self
            .ports
            .insert(txn, port.id.0.to_string(), MapPrelim::default());
        write_port_to_map(txn, &port_map, port);
    }

    fn remove_port(&self, txn: &mut TransactionMut, id: &PortId) {
        self.ports.remove(txn, &id.0.to_string());
    }

    fn insert_edge(&self, txn: &mut TransactionMut, edge: &Edge) {
        let edge_map = MapPrelim::default();
        let edge_ref = self.edges.insert(txn, edge.id.0.to_string(), edge_map);

        edge_ref.insert(txn, "source_port", edge.source_port.0.to_string());
        edge_ref.insert(txn, "target_port", edge.target_port.0.to_string());
    }

    fn remove_edge(&self, txn: &mut TransactionMut, id: &EdgeId) {
        self.edges.remove(txn, &id.0.to_string());
    }

    fn inner_process_intent(&self, txn: &mut TransactionMut, intent: GraphOp) {
        match intent {
            GraphOp::MoveNode { id, x, y } => {
                self.update_node_position(txn, &id, x, y);
            }
            GraphOp::AddNode(node) => self.insert_node(txn, &node),
            GraphOp::RemoveNode { id } => self.remove_node(txn, &id),
            GraphOp::ResizeNode { id, size } => todo!(),
            GraphOp::UpdateNodeData { id, data } => todo!(),
            GraphOp::NodeOrderInsert { id } => self.add_node_order(txn, &id),
            GraphOp::NodeOrderRemove { index } => self.remove_noder_order(txn, index),
            GraphOp::AddPort(port) => self.add_port(txn, &port),
            GraphOp::RemovePort(port_id) => self.remove_port(txn, &port_id),
            GraphOp::AddEdge(edge) => self.insert_edge(txn, &edge),
            GraphOp::RemoveEdge(edge_id) => self.remove_edge(txn, &edge_id),
            GraphOp::Batch(graph_ops) => {
                for op in graph_ops {
                    self.inner_process_intent(txn, op);
                }
            }
        }
    }
}

impl SyncPlugin for YrsSyncPlugin {
    fn name(&self) -> &'static str {
        "YrsSyncPlugin"
    }

    fn setup(&mut self, change_sender: Sender<GraphChange>) {
        let change_sender_clone = change_sender.clone();
        let change_sender_clone2 = change_sender.clone();
        let change_sender_clone3 = change_sender.clone();
        let change_sender_clone4 = change_sender.clone();
        let sub = self.nodes.observe_deep(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == "local_intent".into() => ChangeSource::Local,
                Some(orig) if *orig == "undo_manager".into() => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            for ev in event.iter() {
                if let yrs::types::Event::Map(ev) = ev {
                    let kind = handler_node_change(txn, ev);
                    Runtime::new().unwrap().block_on(async {
                        let _ = change_sender_clone
                            .send(GraphChange {
                                kind: GraphChangeKind::Batch(kind),
                                source,
                            })
                            .await;
                    });
                }
            }
        });

        self._subscription_nodes = Some(sub);

        let sub = self.ports.observe(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == "local_intent".into() => ChangeSource::Local,
                Some(orig) if *orig == "undo_manager".into() => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            for (key, change) in event.keys(txn) {
                if let Some(kind) = parse_port_change(txn, key, change) {
                    Runtime::new().unwrap().block_on(async {
                        let _ = change_sender_clone2
                            .send(GraphChange { kind, source })
                            .await;
                    });
                }
            }
        });
        self._subscription_ports = Some(sub);

        let sub = self.edges.observe(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == "local_intent".into() => ChangeSource::Local,
                Some(orig) if *orig == "undo_manager".into() => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            for (key, change) in event.keys(txn) {
                if let Some(kind) = parse_edge_change(txn, key, change) {
                    Runtime::new().unwrap().block_on(async {
                        let _ = change_sender_clone3
                            .send(GraphChange { kind, source })
                            .await;
                    });
                }
            }
        });
        self._subscription_edges = Some(sub);

        let sub = self.node_order.observe(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == "local_intent".into() => ChangeSource::Local,
                Some(orig) if *orig == "undo_manager".into() => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            let array = event.target();

            let mut list = vec![];
            for item in array.iter(txn) {
                if let Out::Any(Any::String(str)) = item {
                    list.push(NodeId(str.parse().unwrap_or_default()));
                }
            }

            Runtime::new().unwrap().block_on(async {
                let _ = change_sender_clone4
                    .send(GraphChange {
                        kind: GraphChangeKind::NodeOrderUpdate(list),
                        source,
                    })
                    .await;
            });
        });
        self._subscription_order = Some(sub);

        self.from_graph();
    }

    fn process_intent(&self, intent: GraphOp) {
        println!("current op: {:?}", intent.clone());
        let mut txn = self.doc.transact_mut_with(Origin::from("local_intent"));
        self.inner_process_intent(&mut txn, intent);
    }

    fn undo(&mut self) {
        self.undo_manager.undo_blocking();
    }

    fn redo(&mut self) {
        self.undo_manager.redo_blocking();
    }

    fn get_full_snapshot(&self) -> Vec<GraphChange> {
        vec![]
    }
}

fn write_port_to_map(txn: &mut TransactionMut, port_map: &MapRef, port: &Port) {
    port_map.insert(txn, "kind", port.kind.to_string());
    port_map.insert(txn, "node_id", port.node_id.0.to_string());
    port_map.insert(txn, "index", port.index as u32);
    port_map.insert(txn, "position", port.position.to_string());
    port_map.insert(txn, "width", Into::<f32>::into(port.size.width));
    port_map.insert(txn, "height", Into::<f32>::into(port.size.height));
}

fn handler_node_change(
    txn: &yrs::TransactionMut,
    ev: &yrs::types::map::MapEvent,
) -> Vec<GraphChangeKind> {
    let path = ev.path();
    let mut node_id = None;
    if let Some(path) = path.iter().last() {
        if let PathSegment::Key(key) = path {
            node_id = Some(NodeId(key.to_string().parse().unwrap_or_default()))
        }
    }

    let mut x = Out::Any(Any::Null);
    let mut y = Out::Any(Any::Null);
    let mut width = Out::Any(Any::Null);
    let mut height = Out::Any(Any::Null);
    let mut data = Out::Any(Any::Null);
    let mut kind: Vec<GraphChangeKind> = vec![];
    for (key, change) in ev.keys(txn) {
        match parse_node_field_change(key, change) {
            Some((field, value)) => match field.as_str() {
                "x" => x = value,
                "y" => y = value,
                "width" => width = value,
                "height" => height = value,
                "data" => data = value,
                _ => unreachable!(),
            },
            None => {
                if let Some(k) = parse_node_change(txn, key, change) {
                    kind.push(k);
                }
            }
        }
    }

    if let Some(id) = node_id {
        let x = out_f32(txn, x);
        let y = out_f32(txn, y);
        let width = out_f32(txn, width);
        let height = out_f32(txn, height);
        if x > 0.0 || y > 0.0 {
            kind.push(GraphChangeKind::NodeMoved { id, x, y });
        }
        if width > 0.0 {
            kind.push(GraphChangeKind::NodeSetWidthed { id, width })
        }
        if height > 0.0 {
            kind.push(GraphChangeKind::NodeSetHeighted { id, height })
        }

        let data = get_json(txn, data);
        if let Some(data) = data {
            kind.push(GraphChangeKind::NodeDataUpdated { id, data })
        }
    }

    kind
}

fn parse_node_change(
    txn: &yrs::TransactionMut,
    key: &Arc<str>,
    change: &EntryChange,
) -> Option<GraphChangeKind> {
    let id = NodeId(key.to_string().parse().ok()?);

    match change {
        EntryChange::Inserted(value) => {
            if let yrs::Out::YMap(node_map) = value {
                Some(GraphChangeKind::NodeAdded(read_node_from_map(
                    txn, node_map, id,
                )))
            } else {
                None
            }
        }
        EntryChange::Removed(_) => Some(GraphChangeKind::NodeRemoved { id }),
        EntryChange::Updated(_, _) => None,
    }
}

fn parse_node_field_change(key: &Arc<str>, change: &EntryChange) -> Option<(String, Out)> {
    let string = key.to_string();
    let field = string.as_str();
    match (field, change) {
        ("x", EntryChange::Updated(_, new_value)) => Some(("x".into(), new_value.clone())),
        ("y", EntryChange::Updated(_, new_value)) => Some(("y".into(), new_value.clone())),
        ("width", EntryChange::Updated(_, new_value)) => Some(("width".into(), new_value.clone())),
        ("height", EntryChange::Updated(_, new_value)) => {
            Some(("height".into(), new_value.clone()))
        }
        ("data", EntryChange::Updated(_, new_value)) => Some(("data".into(), new_value.clone())),
        _ => None,
    }
}

fn read_node_from_map(txn: &yrs::TransactionMut, node_map: &MapRef, id: NodeId) -> Node {
    let node_type: String = node_map.get_as(txn, "type").unwrap_or_default();
    let x: f32 = node_map.get_as(txn, "x").unwrap_or_default();
    let y: f32 = node_map.get_as(txn, "y").unwrap_or_default();
    let width: f32 = node_map.get_as(txn, "width").unwrap_or_default();
    let height: f32 = node_map.get_as(txn, "height").unwrap_or_default();
    let json: String = node_map.get_as(txn, "data").unwrap_or_default();
    let data = serde_json::from_str(&json).unwrap_or_default();

    let out_inputs = node_map.get(txn, "inputs");
    let mut inputs = vec![];
    if let Some(Out::YArray(arr)) = out_inputs {
        for item in arr.iter(txn) {
            if let Out::Any(Any::String(str)) = item {
                inputs.push(PortId(str.to_string().parse().unwrap_or_default()));
            }
        }
    }

    let out_outputs = node_map.get(txn, "outputs");
    let mut outputs = vec![];
    if let Some(Out::YArray(arr)) = out_outputs {
        for item in arr.iter(txn) {
            if let Out::Any(Any::String(str)) = item {
                outputs.push(PortId(str.to_string().parse().unwrap_or_default()));
            }
        }
    }
    Node {
        id,
        node_type,
        x: px(x),
        y: px(y),
        size: Size::new(px(width), px(height)),
        inputs,
        outputs,
        data,
    }
}

fn parse_port_change(
    txn: &yrs::TransactionMut,
    key: &Arc<str>,
    change: &EntryChange,
) -> Option<GraphChangeKind> {
    let id = PortId(key.to_string().parse().unwrap_or_default());

    match change {
        EntryChange::Inserted(value) => {
            if let yrs::Out::YMap(port_map) = value {
                Some(GraphChangeKind::PortAdded(read_port_from_map(
                    txn, port_map, id,
                )))
            } else {
                None
            }
        }

        EntryChange::Removed(_) => Some(GraphChangeKind::PortRemoved { id }),

        _ => None,
    }
}

fn read_port_from_map(txn: &yrs::TransactionMut, node_map: &MapRef, id: PortId) -> Port {
    let kind: String = node_map.get_as(txn, "kind").unwrap_or_default();
    let node_id: String = node_map.get_as(txn, "node_id").unwrap_or_default();
    let index: u32 = node_map.get_as(txn, "index").unwrap_or_default();
    let position: String = node_map.get_as(txn, "position").unwrap_or_default();
    let width: f32 = node_map.get_as(txn, "width").unwrap_or_default();
    let height: f32 = node_map.get_as(txn, "height").unwrap_or_default();

    Port {
        id,
        kind: if kind == "input" {
            crate::PortKind::Input
        } else {
            crate::PortKind::Output
        },
        index: index as usize,
        node_id: NodeId(node_id.parse().unwrap_or_default()),
        position: crate::PortPosition::from_str(&position).unwrap_or(crate::PortPosition::Left),
        size: Size::new(px(width), px(height)),
    }
}

fn parse_edge_change(
    txn: &yrs::TransactionMut,
    key: &Arc<str>,
    change: &EntryChange,
) -> Option<GraphChangeKind> {
    let id = EdgeId(key.to_string().parse().unwrap_or_default());

    match change {
        EntryChange::Inserted(value) => {
            if let yrs::Out::YMap(edge_map) = value {
                Some(GraphChangeKind::EdgeAdded(read_edge_from_map(
                    txn, edge_map, id,
                )))
            } else {
                None
            }
        }

        EntryChange::Removed(_) => Some(GraphChangeKind::EdgeRemoved { id }),

        _ => None,
    }
}

fn read_edge_from_map(txn: &yrs::TransactionMut, node_map: &MapRef, id: EdgeId) -> Edge {
    let source_port: String = node_map.get_as(txn, "source_port").unwrap_or_default();
    let target_port: String = node_map.get_as(txn, "target_port").unwrap_or_default();

    Edge {
        id,
        source_port: PortId(source_port.parse().unwrap_or_default()),
        target_port: PortId(target_port.parse().unwrap_or_default()),
    }
}

fn get_f32(txn: &yrs::TransactionMut, map: &MapRef, key: &str) -> f32 {
    map.get(txn, key)
        .and_then(|v| v.to_string(txn).parse::<f32>().ok())
        .unwrap_or(0.0)
}

fn out_f32(txn: &yrs::TransactionMut, out: Out) -> f32 {
    out.to_string(txn).parse::<f32>().ok().unwrap_or(0.0)
}

fn get_json(txn: &yrs::TransactionMut, out: Out) -> Option<Value> {
    let str = out.to_string(txn);
    serde_json::from_str(&str).ok()
}
