//! [`Command`](ferrum_flow::Command) for writing node JSON data after execution (or any host logic).

use ferrum_flow::{Command, CommandContext, GraphOp, NodeId};
use serde_json::Value;

/// Replace a node's JSON data through the canvas history / sync pipeline.
pub struct UpdateNodeDataCommand {
    node_id: NodeId,
    new_data: Value,
    old_data: Option<Value>,
}

impl UpdateNodeDataCommand {
    pub fn new(node_id: NodeId, new_data: Value) -> Self {
        Self {
            node_id,
            new_data,
            old_data: None,
        }
    }
}

impl Command for UpdateNodeDataCommand {
    fn name(&self) -> &'static str {
        "update_node_data"
    }

    fn execute(&mut self, ctx: &mut CommandContext) {
        if let Some(node) = ctx.get_node_mut(&self.node_id) {
            self.old_data = Some(node.data_ref().clone());
            node.set_data(self.new_data.clone());
            ctx.port_offset_cache.clear_node(&self.node_id);
        }
    }

    fn undo(&mut self, ctx: &mut CommandContext) {
        if let (Some(old), Some(node)) = (&self.old_data, ctx.get_node_mut(&self.node_id)) {
            node.set_data(old.clone());
            ctx.port_offset_cache.clear_node(&self.node_id);
        }
    }

    fn to_ops(&self, _ctx: &mut CommandContext) -> Vec<GraphOp> {
        vec![GraphOp::UpdateNodeData {
            id: self.node_id,
            data: self.new_data.clone(),
        }]
    }
}
