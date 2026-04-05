use anyhow::Result;
use ferrum_flow::{Node, NodeId, PortId};
use serde_json::Value;
use std::collections::HashMap;

/// Runtime value stored per port.
pub type PortValues = HashMap<PortId, Value>;

/// Context for one graph execution (whole run lifecycle).
pub struct ExecutorContext {
    /// Computed port values (prior outputs plus current node writes).
    pub values: PortValues,
    /// Arbitrary external state (DB handles, config, etc.).
    pub state: HashMap<String, Value>,
}

impl ExecutorContext {
    pub fn get_input(&self, port_id: &PortId) -> Option<&Value> {
        self.values.get(port_id)
    }

    pub fn set_output(&mut self, port_id: PortId, value: Value) {
        self.values.insert(port_id, value);
    }
}

/// Output from executing a single node.
pub struct NodeOutput {
    pub node_id: NodeId,
    pub outputs: PortValues,
    pub error: Option<anyhow::Error>,
}

pub trait NodeProcessor: Send + Sync {
    fn name(&self) -> &str;

    fn execute(&self, node: &Node, ctx: &mut ExecutorContext) -> Result<NodeOutput>;

    fn execute_async<'a>(
        &'a self,
        node: &'a Node,
        ctx: &'a mut ExecutorContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<NodeOutput>> + Send + 'a>> {
        Box::pin(async move { self.execute(node, ctx) })
    }

    /// Whether this node may run in parallel with others (default: true).
    fn is_parallelizable(&self) -> bool {
        true
    }
}
