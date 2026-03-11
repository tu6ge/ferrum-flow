use serde::{Deserialize, Serialize};

use crate::PortId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source_port: PortId,

    pub target_port: PortId,
}

impl Edge {
    pub fn new(id: EdgeId) -> Self {
        Self {
            id,
            source_port: PortId(0),
            target_port: PortId(0),
        }
    }
    pub fn source(mut self, port: PortId) -> Self {
        self.source_port = port;
        self
    }
    pub fn target(mut self, port: PortId) -> Self {
        self.target_port = port;
        self
    }
}
