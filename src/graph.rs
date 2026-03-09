use std::collections::{HashMap, HashSet};

use crate::edge::{Edge, EdgeId};
use crate::node::{Node, NodeId};

#[derive(Debug, Clone)]
pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    node_order: Vec<NodeId>,
    pub edges: HashMap<EdgeId, Edge>,

    pub selected_edge: Option<EdgeId>,
    pub selected_node: HashSet<NodeId>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            node_order: vec![],
            edges: HashMap::new(),
            selected_edge: None,
            selected_node: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        let node_id = node.id.clone();
        self.nodes.insert(node.id, node);
        self.node_order.push(node_id);
    }

    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }

    pub fn node_order(&self) -> &Vec<NodeId> {
        &self.node_order
    }
    pub fn node_order_mut(&mut self) -> &mut Vec<NodeId> {
        &mut self.node_order
    }

    pub fn new_edge(&self) -> Edge {
        let id = self.edges.len() + 1;

        Edge::new(EdgeId(id as u64))
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge.id, edge);
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    #[inline]
    pub fn remove_node(&mut self, id: &NodeId) {
        self.nodes.remove(id);
        let index = self.node_order.iter().position(|v| *v == *id);
        if let Some(index) = index {
            self.node_order.remove(index);
        }
    }

    pub fn add_selected_node(&mut self, id: NodeId, shift: bool) {
        if shift {
            if self.selected_node.contains(&id) {
                self.selected_node.remove(&id);
            } else {
                self.selected_node.insert(id);
            }
        } else {
            self.selected_node.clear();
            self.selected_node.insert(id);
        }
    }
    pub fn clear_selected_node(&mut self) {
        self.selected_node.clear();
    }

    pub fn remove_selected_node(&mut self) -> bool {
        if self.selected_node.len() == 0 {
            return false;
        }

        let mut ids = vec![];
        for id in self.selected_node.iter() {
            ids.push(id.clone());
        }
        for id in ids.iter() {
            self.remove_node(&id);
        }
        self.selected_node.clear();
        return true;
    }
}
