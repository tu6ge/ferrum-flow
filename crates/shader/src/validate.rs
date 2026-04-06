//! Lightweight validation: no node execution, only wiring / semantic hints.

use std::collections::HashSet;

use ferrum_flow::Graph;

fn node_label(graph: &Graph, id: &ferrum_flow::NodeId) -> String {
    graph
        .get_node(id)
        .and_then(|n| n.data.get("label").and_then(|v| v.as_str()))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| graph.get_node(id).map(|n| n.node_type.clone()))
        .unwrap_or_else(|| "node".to_string())
}

/// Lines for logging or UI (non-fatal; save still writes the file).
pub fn graph_validation_notes(graph: &Graph) -> Vec<String> {
    let mut notes = Vec::new();

    let targets: HashSet<ferrum_flow::PortId> = graph
        .edges
        .values()
        .map(|e| e.target_port)
        .collect();

    for (id, node) in graph.nodes() {
        let label = node_label(graph, id);

        for (i, port_id) in node.inputs.iter().enumerate() {
            if !targets.contains(port_id) {
                notes.push(format!(
                    "Unconnected input: port {} of «{label}» (type {})",
                    i + 1,
                    node.node_type,
                ));
            }
        }

        if node.node_type == "output" && node.inputs.is_empty() {
            notes.push(format!(
                "Output node «{label}» has no inputs (graph may be incomplete)"
            ));
        }
    }

    if graph.nodes().is_empty() {
        notes.push("Graph is empty".to_string());
    }

    notes
}
