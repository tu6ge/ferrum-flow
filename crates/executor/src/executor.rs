use std::collections::HashMap;

use ferrum_flow::{Graph, NodeId, PortId};

use crate::{
    context::{ExecutorContext, NodeOutput},
    registry::NodeRegistry,
};

pub struct GraphExecutor {
    registry: NodeRegistry,
    mode: ExecutionMode,
}

#[derive(Default)]
pub enum ExecutionMode {
    #[default]
    Sequential, // Sequential execution; easier to debug
    Parallel, // Run independent nodes concurrently (e.g. rayon/tokio)
}

impl GraphExecutor {
    pub fn new(registry: NodeRegistry) -> Self {
        Self {
            registry,
            mode: ExecutionMode::default(),
        }
    }

    pub fn with_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Topological order and edge wiring for step-by-step execution (see [`Self::execute_node`]).
    pub fn execution_plan(
        &self,
        graph: &Graph,
    ) -> Result<(Vec<NodeId>, HashMap<PortId, PortId>), anyhow::Error> {
        Ok((self.topological_sort(graph)?, self.build_edge_map(graph)))
    }

    /// Run a single node: wire inputs from `edge_map`, invoke handler, store outputs in `ctx`.
    pub fn execute_node(
        &self,
        graph: &Graph,
        node_id: &NodeId,
        ctx: &mut ExecutorContext,
        edge_map: &HashMap<PortId, PortId>,
    ) -> Result<NodeOutput, anyhow::Error> {
        let node = graph
            .get_node(node_id)
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", node_id))?;

        for input_port_id in node.inputs() {
            if let Some(source_port_id) = edge_map.get(input_port_id) {
                if let Some(val) = ctx.values.get(source_port_id).cloned() {
                    ctx.values.insert(*input_port_id, val);
                }
            }
        }

        let handler = self.registry.get(&node.execute_type_ref()).ok_or_else(|| {
            anyhow::anyhow!("No handler for node type: {}", node.execute_type_ref())
        })?;

        let output = handler.execute(node, ctx)?;

        for (port_id, value) in &output.outputs {
            ctx.values.insert(*port_id, value.clone());
        }

        Ok(output)
    }

    /// Executes the full graph and returns each node's output.
    pub fn run(
        &self,
        graph: &Graph,
        initial_ctx: ExecutorContext,
    ) -> Result<Vec<NodeOutput>, anyhow::Error> {
        let (order, edge_map) = self.execution_plan(graph)?;
        let mut ctx = initial_ctx;
        let mut results = Vec::new();

        for node_id in &order {
            results.push(self.execute_node(graph, node_id, &mut ctx, &edge_map)?);
        }

        Ok(results)
    }

    /// Topological sort (Kahn's algorithm).
    fn topological_sort(&self, graph: &Graph) -> Result<Vec<NodeId>, anyhow::Error> {
        // In-degree count per node
        let mut in_degree: HashMap<NodeId, usize> =
            graph.nodes().keys().map(|id| (*id, 0)).collect();

        // Reverse lookup: port -> owning node
        let port_to_node: HashMap<PortId, NodeId> = graph
            .ports_values()
            .map(|p| (p.id(), p.node_id()))
            .collect();

        // Derive dependencies from edges
        for edge in graph.edges_values() {
            if let Some(target_node) = port_to_node.get(&edge.target_port) {
                *in_degree.entry(*target_node).or_insert(0) += 1;
            }
        }

        let mut queue: std::collections::VecDeque<NodeId> = in_degree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(&id, _)| id)
            .collect();

        // Queue order yields a stable topological order
        let mut sorted = Vec::new();
        while let Some(node_id) = queue.pop_front() {
            sorted.push(node_id);
            for edge in graph.edges_values() {
                if let Some(&src_node) = port_to_node.get(&edge.source_port) {
                    if src_node == node_id {
                        if let Some(&tgt_node) = port_to_node.get(&edge.target_port) {
                            let d = in_degree.get_mut(&tgt_node).unwrap();
                            *d -= 1;
                            if *d == 0 {
                                queue.push_back(tgt_node);
                            }
                        }
                    }
                }
            }
        }

        if sorted.len() != graph.nodes().len() {
            return Err(anyhow::anyhow!("Graph contains a cycle"));
        }
        Ok(sorted)
    }

    fn build_edge_map(&self, graph: &Graph) -> HashMap<PortId, PortId> {
        graph
            .edges_values()
            .map(|e| (e.target_port, e.source_port))
            .collect()
    }
}
