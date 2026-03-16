use std::collections::HashSet;

use crate::{EdgeId, NodeId, canvas::Command, plugin::PluginContext};

pub struct SelecteNode {
    node_id: NodeId,
    shift: bool,
    old_node_order: Vec<NodeId>,
    old_selected_edge: HashSet<EdgeId>,
    old_selected_node: HashSet<NodeId>,
}

impl SelecteNode {
    pub fn new(node_id: NodeId, shift: bool, ctx: &PluginContext) -> Self {
        Self {
            node_id,
            shift,
            old_node_order: ctx.graph.node_order().clone(),
            old_selected_edge: ctx.graph.selected_edge.clone(),
            old_selected_node: ctx.graph.selected_node.clone(),
        }
    }
}

impl Command for SelecteNode {
    fn name(&self) -> &'static str {
        "select_node"
    }
    fn execute(&mut self, ctx: &mut crate::canvas::CanvasState) {
        if !self.shift {
            ctx.graph.clear_selected_edge();
        }
        ctx.graph.add_selected_node(self.node_id, self.shift);
        ctx.graph.bring_node_to_front(self.node_id);
    }
    fn undo(&mut self, ctx: &mut crate::canvas::CanvasState) {
        ctx.graph.selected_node = self.old_selected_node.clone();
        ctx.graph.selected_edge = self.old_selected_edge.clone();
        let a = ctx.graph.node_order_mut();
        *a = self.old_node_order.clone();
    }
}
