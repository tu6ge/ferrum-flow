use gpui::{Pixels, Point};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub x: Pixels,
    pub y: Pixels,

    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

impl Node {
    pub fn new(id: u64, x: f32, y: f32) -> Self {
        Self {
            id: NodeId(id),
            x: x.into(),
            y: y.into(),
            inputs: vec![],
            outputs: vec![],
        }
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
