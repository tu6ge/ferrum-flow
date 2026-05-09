use std::collections::{HashMap, HashSet};

use gpui::{Pixels, Point, px};

use crate::{CompositeCommand, Edge, Graph, plugin::PluginContext};

use super::copied_subgraph::CopiedSubgraph;
use crate::plugins::{CreateEdge, CreateNode, CreatePort};

#[derive(Clone)]
pub(crate) struct ClipboardShared(pub CopiedSubgraph);

pub(crate) fn set_clipboard_subgraph(ctx: &mut PluginContext, sub: CopiedSubgraph) {
    ctx.shared_state.insert(ClipboardShared(sub));
}

pub(crate) fn get_clipboard_subgraph(ctx: &PluginContext) -> Option<CopiedSubgraph> {
    ctx.shared_state
        .get::<ClipboardShared>()
        .map(|s| s.0.clone())
}

pub(crate) fn has_clipboard_subgraph(ctx: &PluginContext) -> bool {
    ctx.shared_state.contains::<ClipboardShared>()
}

pub(crate) fn extract_subgraph(graph: &Graph) -> Option<CopiedSubgraph> {
    if graph.selected_node_is_empty() {
        return None;
    }
    let node_ids = graph.selected_node();
    if node_ids.is_empty() {
        return None;
    }

    let mut port_ids = HashSet::new();
    let mut nodes = Vec::with_capacity(node_ids.len());
    let mut ports = Vec::new();
    for nid in node_ids {
        let n = graph.get_node(nid)?;
        for pid in n.inputs().iter().chain(n.outputs().iter()) {
            port_ids.insert(*pid);
            if let Some(p) = graph.get_port(pid) {
                ports.push(p.clone());
            }
        }
        nodes.push(n.clone());
    }

    let edges = graph
        .edges_values()
        .filter(|e| port_ids.contains(&e.source_port) && port_ids.contains(&e.target_port))
        .cloned()
        .collect();

    Some(CopiedSubgraph {
        nodes,
        ports,
        edges,
    })
}

/// Top-left of the axis-aligned bounding box of copied node positions (world space).
fn subgraph_bounds_top_left(sub: &CopiedSubgraph) -> Point<Pixels> {
    let (mut min_x, mut min_y) = (f32::INFINITY, f32::INFINITY);
    for n in &sub.nodes {
        let (x, y) = n.position();
        min_x = min_x.min(x.into());
        min_y = min_y.min(y.into());
    }
    Point::new(px(min_x), px(min_y))
}

/// Paste with the subgraph's bounding-box top-left placed at `anchor_world`.
pub(crate) fn paste_subgraph_at_world(
    ctx: &mut PluginContext,
    sub: &CopiedSubgraph,
    anchor_world: Point<Pixels>,
) {
    paste_subgraph_with_anchor(ctx, sub, anchor_world);
}

/// Paste offset from the copied layout (keyboard paste): bbox top-left moves by (40, 40) in world space.
pub(crate) fn paste_subgraph(ctx: &mut PluginContext, sub: &CopiedSubgraph) {
    const NUDGE: f32 = 40.0;
    let origin = subgraph_bounds_top_left(sub);
    let anchor = Point::new(origin.x + px(NUDGE), origin.y + px(NUDGE));
    paste_subgraph_with_anchor(ctx, sub, anchor);
}

fn paste_subgraph_with_anchor(
    ctx: &mut PluginContext,
    sub: &CopiedSubgraph,
    anchor_world: Point<Pixels>,
) {
    if sub.nodes.is_empty() {
        return;
    }

    let origin = subgraph_bounds_top_left(sub);
    let ox: f32 = origin.x.into();
    let oy: f32 = origin.y.into();
    let ax: f32 = anchor_world.x.into();
    let ay: f32 = anchor_world.y.into();

    let mut node_map = HashMap::new();
    for n in &sub.nodes {
        node_map.insert(n.id(), ctx.graph.next_node_id());
    }
    let mut port_map = HashMap::new();
    for p in &sub.ports {
        port_map.insert(p.id(), ctx.graph.next_port_id());
    }

    let mut composite = CompositeCommand::new();

    let mut new_node_ids = Vec::new();

    for old in &sub.nodes {
        let new_id = node_map[&old.id()];
        let (x, y) = old.position();
        let nx = ax + f32::from(x) - ox;
        let ny = ay + f32::from(y) - oy;
        // Clone-then-patch avoids missing newly added fields on Node.
        let mut node = old.clone();
        node.set_id(new_id);
        node.set_position(nx.into(), ny.into());
        node.clear_port_refs();

        for pid in old.inputs() {
            node.push_input(port_map[pid]);
        }
        for pid in old.outputs() {
            node.push_output(port_map[pid]);
        }
        new_node_ids.push(new_id);
        composite.push(CreateNode::new(node));
    }

    for old in &sub.ports {
        // Clone-then-patch avoids missing newly added fields on Port.
        let mut port = old.clone();
        port.set_id(port_map[&old.id()]);
        port.set_node_id(node_map[&old.node_id()]);
        composite.push(CreatePort::new(port));
    }

    for old in &sub.edges {
        let edge = Edge {
            id: ctx.graph.next_edge_id(),
            source_port: port_map[&old.source_port],
            target_port: port_map[&old.target_port],
        };
        composite.push(CreateEdge::new(edge));
    }

    let pasted_ids = sub.nodes.iter().map(|n| node_map[&n.id()]);

    ctx.execute_command(composite);
    ctx.clear_selected_edge();
    ctx.clear_selected_node();
    for nid in pasted_ids {
        ctx.add_selected_node(nid, true);
    }
    ctx.cache_port_offset_with_node(&new_node_ids);
}

#[cfg(test)]
mod tests {
    use gpui::{Point, px};
    use serde_json::{Value, json};

    use crate::plugin_testing::PluginTestHarness;

    use super::{extract_subgraph, paste_subgraph_at_world};

    fn node_for_compare(mut value: Value) -> Value {
        let obj = value
            .as_object_mut()
            .expect("serialized node should be JSON object");
        obj.remove("id");
        obj.remove("inputs");
        obj.remove("outputs");
        value
    }

    fn port_for_compare(mut value: Value) -> Value {
        let obj = value
            .as_object_mut()
            .expect("serialized port should be JSON object");
        obj.remove("id");
        obj.remove("node_id");
        value
    }

    #[test]
    fn paste_keeps_node_fields_except_ids_and_port_refs() {
        let mut harness = PluginTestHarness::default();

        let n1 = harness
            .graph
            .create_node("default")
            .execute_type("exec-a")
            .position(120.0, 80.0)
            .size(210.0, 90.0)
            .input()
            .output()
            .data(json!({"label":"A","tag":"node-a","nested":{"k":1}}))
            .build();
        let n2 = harness
            .graph
            .create_node("default")
            .execute_type("exec-b")
            .position(360.0, 220.0)
            .size(260.0, 110.0)
            .input()
            .output()
            .data(json!({"label":"B","tag":"node-b","nested":{"k":2}}))
            .build();

        harness.graph.add_selected_node(n1, false);
        harness.graph.add_selected_node(n2, true);

        let copied = extract_subgraph(&harness.graph).expect("copy should extract selected nodes");
        let old_a = harness.graph.get_node(&n1).expect("original node-a exists");
        let old_b = harness.graph.get_node(&n2).expect("original node-b exists");
        let (x1, y1) = old_a.position();
        let (x2, y2) = old_b.position();
        let min_x = f32::from(x1).min(f32::from(x2));
        let min_y = f32::from(y1).min(f32::from(y2));
        let anchor = Point::new(px(min_x), px(min_y));

        harness.with_plugin_context(|ctx| {
            paste_subgraph_at_world(ctx, &copied, anchor);
        });

        let pasted_ids: Vec<_> = harness.graph.selected_node().iter().copied().collect();
        assert_eq!(
            pasted_ids.len(),
            2,
            "paste should select exactly two new nodes"
        );

        let mut pasted_a = None;
        let mut pasted_b = None;
        for id in pasted_ids {
            let node = harness
                .graph
                .get_node(&id)
                .expect("pasted node should exist");
            match node.data_ref().get("tag").and_then(Value::as_str) {
                Some("node-a") => pasted_a = Some(node.clone()),
                Some("node-b") => pasted_b = Some(node.clone()),
                _ => {}
            }
        }

        let pasted_a = pasted_a.expect("pasted node-a should exist");
        let pasted_b = pasted_b.expect("pasted node-b should exist");
        let old_a = harness.graph.get_node(&n1).expect("original node-a exists");
        let old_b = harness.graph.get_node(&n2).expect("original node-b exists");

        assert_eq!(old_a.inputs().len(), pasted_a.inputs().len());
        assert_eq!(old_a.outputs().len(), pasted_a.outputs().len());
        assert_eq!(old_b.inputs().len(), pasted_b.inputs().len());
        assert_eq!(old_b.outputs().len(), pasted_b.outputs().len());

        for (old_pid, pasted_pid) in old_a.inputs().iter().zip(pasted_a.inputs().iter()) {
            let old_port = harness
                .graph
                .get_port(old_pid)
                .expect("old node-a input port should exist");
            let pasted_port = harness
                .graph
                .get_port(pasted_pid)
                .expect("pasted node-a input port should exist");
            assert_eq!(
                port_for_compare(
                    serde_json::to_value(old_port).expect("serialize old node-a input")
                ),
                port_for_compare(
                    serde_json::to_value(pasted_port).expect("serialize pasted node-a input")
                ),
                "node-a input port fields should remain consistent after copy/paste"
            );
        }
        for (old_pid, pasted_pid) in old_a.outputs().iter().zip(pasted_a.outputs().iter()) {
            let old_port = harness
                .graph
                .get_port(old_pid)
                .expect("old node-a output port should exist");
            let pasted_port = harness
                .graph
                .get_port(pasted_pid)
                .expect("pasted node-a output port should exist");
            assert_eq!(
                port_for_compare(
                    serde_json::to_value(old_port).expect("serialize old node-a output")
                ),
                port_for_compare(
                    serde_json::to_value(pasted_port).expect("serialize pasted node-a output")
                ),
                "node-a output port fields should remain consistent after copy/paste"
            );
        }
        for (old_pid, pasted_pid) in old_b.inputs().iter().zip(pasted_b.inputs().iter()) {
            let old_port = harness
                .graph
                .get_port(old_pid)
                .expect("old node-b input port should exist");
            let pasted_port = harness
                .graph
                .get_port(pasted_pid)
                .expect("pasted node-b input port should exist");
            assert_eq!(
                port_for_compare(
                    serde_json::to_value(old_port).expect("serialize old node-b input")
                ),
                port_for_compare(
                    serde_json::to_value(pasted_port).expect("serialize pasted node-b input")
                ),
                "node-b input port fields should remain consistent after copy/paste"
            );
        }
        for (old_pid, pasted_pid) in old_b.outputs().iter().zip(pasted_b.outputs().iter()) {
            let old_port = harness
                .graph
                .get_port(old_pid)
                .expect("old node-b output port should exist");
            let pasted_port = harness
                .graph
                .get_port(pasted_pid)
                .expect("pasted node-b output port should exist");
            assert_eq!(
                port_for_compare(
                    serde_json::to_value(old_port).expect("serialize old node-b output")
                ),
                port_for_compare(
                    serde_json::to_value(pasted_port).expect("serialize pasted node-b output")
                ),
                "node-b output port fields should remain consistent after copy/paste"
            );
        }

        assert_eq!(
            node_for_compare(serde_json::to_value(old_a).expect("serialize old node-a")),
            node_for_compare(serde_json::to_value(&pasted_a).expect("serialize pasted node-a")),
            "node-a fields should remain consistent after copy/paste"
        );
        assert_eq!(
            node_for_compare(serde_json::to_value(old_b).expect("serialize old node-b")),
            node_for_compare(serde_json::to_value(&pasted_b).expect("serialize pasted node-b")),
            "node-b fields should remain consistent after copy/paste"
        );
    }
}
