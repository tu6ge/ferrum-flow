use std::sync::{Arc, mpsc::Sender};

use yrs::{
    Array as _, ArrayRef, Doc, GetString as _, Map, MapPrelim, MapRef, Observable as _, Out,
    Transact, TransactionMut, types::EntryChange,
};

use crate::{
    ChangeSource, Edge, EdgeId, GraphChange, GraphChangeKind, GraphOp, Node, NodeId, Port, PortId,
    SyncPlugin,
};

pub struct YrsSyncPlugin {
    doc: yrs::Doc,
    graph: MapRef,
    pub nodes: MapRef,        // ref HashMap<NodeId, Node>
    pub ports: MapRef,        // ref HashMap<PortId, Port>
    pub edges: MapRef,        // ref HashMap<EdgeId, Edge>
    pub node_order: ArrayRef, // ref Vec<NodeId>
    undo_manager: yrs::UndoManager,
    _subscription: Option<yrs::Subscription>,
}

impl YrsSyncPlugin {
    pub fn new() -> Self {
        let doc = Doc::new();
        let root = doc.get_or_insert_map("graph");
        let mut txn = doc.transact_mut();
        let nodes = root.get_or_init(&mut txn, "nodes");
        let ports = root.get_or_init(&mut txn, "ports");
        let edges = root.get_or_init(&mut txn, "edges");
        let node_order = root.get_or_init(&mut txn, "node_order");
        drop(txn);

        Self {
            undo_manager: yrs::UndoManager::new(&doc, &root),
            doc,
            graph: root,
            nodes,
            ports,
            edges,
            node_order,
            _subscription: None,
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

        let data_json = serde_json::to_string(&node.data).unwrap_or_default();
        node_ref.insert(txn, "data", data_json);

        self.node_order.push_back(txn, node.id.0.to_string());
    }

    fn update_node_position(&self, txn: &mut TransactionMut, id: &NodeId, x: f32, y: f32) {
        if let Some(yrs::Out::YMap(node_ref)) = self.nodes.get(txn, &id.0.to_string()) {
            node_ref.try_update(txn, "x", x);
            node_ref.try_update(txn, "y", y);
        }
    }

    fn node_to_front(&self, txn: &mut TransactionMut, id: &NodeId) {
        let Some(index) = self.node_order.iter(txn).position(|i| {
            if let yrs::Out::YText(inner) = i {
                inner.get_string(txn) == id.0.to_string()
            } else {
                return false;
            }
        }) else {
            return;
        };
        self.node_order.remove(txn, index as u32);
        self.node_order.push_back(txn, id.0.to_string());
    }

    fn remove_node(&self, txn: &mut TransactionMut, id: &NodeId) {
        self.nodes.remove(txn, &id.0.to_string());

        let Some(index) = self.node_order.iter(txn).position(|i| {
            if let yrs::Out::YText(inner) = i {
                inner.get_string(txn) == id.0.to_string()
            } else {
                return false;
            }
        }) else {
            return;
        };
        self.node_order.remove(txn, index as u32);
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
            GraphOp::NodeToFront { id } => self.node_to_front(txn, &id),
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
        let sub = self.graph.observe(move |txn, event| {
            let source = match txn.origin() {
                Some(orig) if *orig == "local_intent".into() => ChangeSource::Local,
                Some(orig) if *orig == "undo_manager".into() => ChangeSource::Undo,
                _ => ChangeSource::Remote,
            };

            for (key, change) in event.keys(txn) {
                let kind = parse_yrs_change_to_kind(key, change);

                let _ = change_sender.send(GraphChange { kind, source });
            }
        });

        self._subscription = Some(sub);
    }

    fn process_intent(&self, intent: GraphOp) {
        let mut txn = self.doc.transact_mut_with("local_intent");
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

fn parse_yrs_change_to_kind(key: &Arc<str>, change: &EntryChange) -> GraphChangeKind {
    todo!()
}

fn write_port_to_map(txn: &mut TransactionMut, port_map: &MapRef, port: &Port) {
    port_map.insert(txn, "kind", port.kind.to_string());
    port_map.insert(txn, "index", port.index as u32);
    port_map.insert(txn, "position", port.position.to_string());
    port_map.insert(txn, "width", Into::<f32>::into(port.size.width));
    port_map.insert(txn, "height", Into::<f32>::into(port.size.height));
}
