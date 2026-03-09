use serde::{Deserialize, Serialize};

use crate::node::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source_node: NodeId,
    pub source_port: String,

    pub target_node: NodeId,
    pub target_port: String,
}

impl Edge {
    pub fn new(id: EdgeId) -> Self {
        Self {
            id,
            source_node: NodeId(0),
            source_port: "".into(),
            target_node: NodeId(0),
            target_port: "".into(),
        }
    }
    pub fn source(mut self, node_id: NodeId, port: impl Into<String>) -> Self {
        self.source_node = node_id;
        self.source_port = port.into();
        self
    }
    pub fn target(mut self, node_id: NodeId, port: impl Into<String>) -> Self {
        self.target_node = node_id;
        self.target_port = port.into();
        self
    }
}
