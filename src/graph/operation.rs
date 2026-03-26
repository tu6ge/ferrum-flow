use crate::{Edge, Node, NodeId};

use super::Graph;

use gpui::px;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub trait Operation: Serialize + DeserializeOwned {
    fn apply(&self, graph: &mut Graph);
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GraphOp {
    AddNode { node: Node },
    RemoveNode { node_id: NodeId },
    MoveNode { node_id: NodeId, x: f32, y: f32 },
    AddEdge { edge: Edge },
}

impl Operation for GraphOp {
    fn apply(&self, graph: &mut Graph) {
        match self {
            GraphOp::AddNode { node } => {
                graph.nodes.insert(node.id, node.clone());
            }
            GraphOp::RemoveNode { node_id } => {
                graph.nodes.remove(node_id);
            }
            GraphOp::MoveNode { node_id, x, y } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    node.x = px(*x);
                    node.y = px(*y);
                }
            }
            _ => unimplemented!(),
        }
    }
}
