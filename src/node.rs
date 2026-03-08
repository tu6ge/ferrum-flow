use std::fmt::Display;

use gpui::{Pixels, Point};
use serde::{Deserialize, Serialize};
use serde_json::json;

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

    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
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
    pub fn output(mut self, id: String, point: Point<Pixels>) -> Self {
        self.outputs.push(Port {
            id,
            kind: PortKind::Output,
            point,
        });
        self
    }

    pub fn input(mut self, id: String, point: Point<Pixels>) -> Self {
        self.inputs.push(Port {
            id,
            kind: PortKind::Input,
            point,
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: String,
    pub kind: PortKind,
    pub point: Point<Pixels>,
}
