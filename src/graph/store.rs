use gpui::{Pixels, Size};
use serde::{Deserialize, Serialize};

use crate::{Edge, EdgeId, Graph, Node, NodeId, Port, PortId};

pub trait GraphStore {
    fn get_graph(&self) -> Graph;

    fn apply_op(&mut self, op: GraphOp);

    fn subscribe(&mut self, f: Box<dyn FnMut(&GraphChange)>);
}

pub struct LocalGraphStore {
    graph: Graph,
    listeners: Vec<Box<dyn FnMut(&GraphChange)>>,
}

// pub struct YrsGraphStore {
//     doc: yrs::Doc,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphOp {
    // --- Node ---
    AddNode(Node),

    RemoveNode { id: NodeId },

    MoveNode { id: NodeId, x: f32, y: f32 },

    ResizeNode { id: NodeId, size: Size<Pixels> },

    UpdateNodeData { id: NodeId, data: serde_json::Value },

    // --- Port ---
    AddPort(Port),

    RemovePort(PortId),

    // --- Edge ---
    AddEdge(Edge),

    RemoveEdge(EdgeId),

    Batch(Vec<GraphOp>),
}

#[derive(Debug, Clone)]
pub struct GraphChange {
    kind: GraphChangeKind,
    source: ChangeSource,
}

impl GraphChange {
    pub fn is_local(&self) -> bool {
        matches!(self.source, ChangeSource::Local)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeSource {
    Local,
    Remote,
    Undo,
    Redo,
}

#[derive(Debug, Clone)]
pub enum GraphChangeKind {
    // --- Node ---
    NodeAdded(Node),
    NodeRemoved {
        id: NodeId,
    },
    NodeMoved {
        id: NodeId,
        old_pos: (f32, f32),
        new_pos: (f32, f32),
    },
    NodeResized {
        id: NodeId,
        size: Size<Pixels>,
    },
    NodeDataUpdated {
        id: NodeId,
        data: serde_json::Value,
    },

    // --- Port ---
    PortAdded(Port),
    PortRemoved {
        id: PortId,
    },

    // --- Edge ---
    EdgeAdded(Edge),
    EdgeRemoved {
        id: EdgeId,
    },

    Batch(Vec<GraphChangeKind>),
}

impl GraphStore for LocalGraphStore {
    fn apply_op(&mut self, op: GraphOp) {
        let change = self.apply_to_graph(op);

        for l in &mut self.listeners {
            l(&change);
        }
    }
    fn get_graph(&self) -> Graph {
        self.graph.clone()
    }
    fn subscribe(&mut self, f: Box<dyn FnMut(&GraphChange)>) {
        self.listeners.push(f);
    }
}

impl LocalGraphStore {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            listeners: vec![],
        }
    }

    fn apply_to_graph(&mut self, op: GraphOp) -> GraphChange {
        todo!()
    }
}
