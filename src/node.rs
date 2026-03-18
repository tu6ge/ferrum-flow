use std::fmt::Display;

use gpui::{Bounds, Pixels, Point, Size, px};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::Graph;

pub const DEFAULT_NODE_WIDTH: Pixels = px(120.0);
pub const DEFAULT_NODE_HEIGHT: Pixels = px(60.0);

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
    pub size: Size<Pixels>,

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
            size: Size {
                width: DEFAULT_NODE_WIDTH,
                height: DEFAULT_NODE_HEIGHT,
            },
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
        Bounds::new(self.point(), self.size)
    }

    pub fn set_size(mut self, size: Size<Pixels>) -> Self {
        self.size = size;
        self
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

pub struct NodeBuilder {
    node_type: String,
    x: Pixels,
    y: Pixels,
    size: Size<Pixels>,
    input_count: usize,
    output_count: usize,
    data: serde_json::Value,
}

impl NodeBuilder {
    pub fn new(node_type: impl Into<String>) -> Self {
        Self {
            node_type: node_type.into(),
            x: px(0.0),
            y: px(0.0),
            size: Size {
                width: DEFAULT_NODE_WIDTH,
                height: DEFAULT_NODE_HEIGHT,
            },
            input_count: 0,
            output_count: 0,
            data: serde_json::Value::Null,
        }
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.x = x.into();
        self.y = y.into();
        self
    }

    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.size = Size {
            width: w.into(),
            height: h.into(),
        };
        self
    }

    pub fn input(mut self) -> Self {
        self.input_count += 1;
        self
    }

    pub fn output(mut self) -> Self {
        self.output_count += 1;
        self
    }

    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    pub fn build(self, graph: &mut Graph) -> NodeId {
        let node_id = graph.next_node_id();

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // 创建 input ports
        for i in 0..self.input_count {
            let port_id = graph.next_port_id();

            graph.ports.insert(
                port_id,
                Port {
                    id: port_id,
                    kind: PortKind::Input,
                    index: i,
                    node_id,
                },
            );

            inputs.push(port_id);
        }

        // 创建 output ports
        for i in 0..self.output_count {
            let port_id = graph.next_port_id();

            graph.ports.insert(
                port_id,
                Port {
                    id: port_id,
                    kind: PortKind::Output,
                    index: i,
                    node_id,
                },
            );

            outputs.push(port_id);
        }

        let node = Node {
            id: node_id,
            node_type: self.node_type,
            x: self.x,
            y: self.y,
            size: self.size,
            inputs,
            outputs,
            data: self.data,
        };

        graph.nodes.insert(node_id, node);
        let order = graph.node_order_mut();
        order.push(node_id);

        node_id
    }
}
