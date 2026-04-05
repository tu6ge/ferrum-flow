use std::collections::HashMap;

use ferrum_flow::{Graph, Node};
use ferrum_flow_executor::{
    ExecutionMode, ExecutorContext, GraphExecutor, NodeHandler, NodeOutput, NodeRegistry,
    PortValues,
};
use serde_json::Value;

pub struct AddNumbersHandler;

impl NodeHandler for AddNumbersHandler {
    fn name(&self) -> &str {
        "add_numbers"
    }

    fn execute(&self, node: &Node, ctx: &mut ExecutorContext) -> anyhow::Result<NodeOutput> {
        // 从 node.data 或 input ports 读取参数
        let a = node
            .inputs
            .get(0)
            .and_then(|p| ctx.get_input(p))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let b = node
            .inputs
            .get(1)
            .and_then(|p| ctx.get_input(p))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let mut outputs = PortValues::new();
        if let Some(out_port) = node.outputs.first() {
            outputs.insert(*out_port, Value::from(a + b));
        }

        Ok(NodeOutput {
            node_id: node.id,
            outputs,
            error: None,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let mut registry = NodeRegistry::new();
    registry.register(AddNumbersHandler);

    let executor = GraphExecutor::new(registry).with_mode(ExecutionMode::Sequential);

    let ctx = ExecutorContext {
        values: HashMap::new(),
        state: HashMap::new(),
    };

    let graph = Graph::new();

    let results = executor.run(&graph, ctx)?;
    Ok(())
}
