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
