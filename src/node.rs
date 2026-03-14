use std::fmt::Display;

use gpui::{Bounds, Pixels, Point, Size};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::canvas::{DEFAULT_NODE_HEIGHT, DEFAULT_NODE_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: String,
    pub x: Pixels,
    pub y: Pixels,

    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub data: serde_json::Value,
}

impl Node {
    pub fn new(id: u64, x: f32, y: f32) -> Self {
        Self {
            id: NodeId(id),
            node_type: String::new(),
            x: x.into(),
            y: y.into(),
            inputs: vec![],
            outputs: vec![],
            data: json!({}),
        }
    }

    pub fn node_type(mut self, ty: impl Into<String>) -> Self {
        self.node_type = ty.into();
        self
    }

    pub fn point(&self) -> Point<Pixels> {
        Point::new(self.x, self.y)
    }

    pub fn bounds(&self) -> Bounds<Pixels> {
        Bounds::new(
            self.point(),
            Size::new(DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT),
        )
    }

    pub fn output(mut self, id: PortId) -> Self {
        self.outputs.push(id);
        self
    }

    pub fn input(mut self, id: PortId) -> Self {
        self.inputs.push(id);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub kind: PortKind,
    pub index: usize,
    pub node_id: NodeId,
}

impl Port {
    pub fn new_input(id: u64, node_id: u64, index: usize) -> Self {
        Self {
            id: PortId(id),
            kind: PortKind::Input,
            index,
            node_id: NodeId(node_id),
        }
    }
    pub fn new_output(id: u64, node_id: u64, index: usize) -> Self {
        Self {
            id: PortId(id),
            kind: PortKind::Output,
            index,
            node_id: NodeId(node_id),
        }
    }
}
