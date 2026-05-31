//! Standalone node + edge paint ([`GraphPlugin`]); replaces [`crate::plugins::NodePlugin`] and
//! [`crate::plugins::EdgePlugin`] on the same canvas (do not register those together).
//!
//! Flat graphs: one node layer + edge overlay. Nested graphs: group z-order (shell → intra edges →
//! children → ports) plus cross-parent edge overlay.
//!
//! Z-order per group anchor (product rules):
//! 1. Parent shell (background)
//! 2. Intra-parent edges (same direct parent; below children)
//! 3. Children (nested group or leaf cards)
//! 4. Parent ports
//! Cross-parent edges are painted in a top overlay within this layer.

mod hierarchy;
mod plan;

use std::collections::{HashMap, HashSet};

use gpui::{
    AnyElement, Element as _, ElementId, InteractiveElement as _, ParentElement as _, Styled as _,
    div,
};

use crate::EdgeId;
use crate::plugin::{Plugin, RenderContext, RenderLayer};
use crate::plugins::edge::{EdgeGeometry, edge_geometry2, edges_canvas_element};
use crate::plugins::node::{
    ActiveNodeDrag, node_ids_for_drag_overlay, render_node_cards, render_node_ports,
    render_node_shell,
};

use hierarchy::GraphHierarchy;
pub use plan::{EdgePaintKind, classify_edge};

pub struct GraphPlugin;

impl GraphPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GraphPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for GraphPlugin {
    fn name(&self) -> &'static str {
        "graph"
    }

    fn priority(&self) -> i32 {
        55
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Nodes
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<AnyElement> {
        let drag_overlay: HashSet<_> = ctx
            .get_shared_state::<ActiveNodeDrag>()
            .map(|d| {
                node_ids_for_drag_overlay(ctx.graph, d.0.as_ref())
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default();

        if !ctx.graph.has_node_hierarchy() {
            return render_flat_graph(ctx, &drag_overlay);
        }

        let stroke = ctx.theme.edge_stroke;
        let stroke_sel = ctx.theme.edge_stroke_selected;
        let selected = ctx.graph.selected_edge().clone();

        let mut intra_by_parent: HashMap<crate::NodeId, Vec<(EdgeId, Option<EdgeGeometry>)>> =
            HashMap::new();
        let mut cross_edges: Vec<(EdgeId, Option<EdgeGeometry>)> = Vec::new();

        for (_, edge) in ctx.graph.edges().iter() {
            if !plan::edge_is_visible(ctx, edge) {
                continue;
            }
            ctx.cache_port_offset_with_edge(&edge.id);
            let geom = edge_geometry2(edge, ctx);
            match classify_edge(ctx.graph, edge) {
                EdgePaintKind::IntraParent { parent } => {
                    intra_by_parent
                        .entry(parent)
                        .or_default()
                        .push((edge.id, geom));
                }
                EdgePaintKind::Cross => cross_edges.push((edge.id, geom)),
            }
        }

        let paint_order = ctx.graph.paint_order();
        let mut body_children: Vec<AnyElement> = Vec::new();
        let mut covered = HashSet::new();

        // DOM order follows `paint_order` (roots / siblings / click-to-front), not
        // "all groups first, then root leaves".
        for &id in &paint_order {
            if covered.contains(&id) || drag_overlay.contains(&id) {
                continue;
            }
            if !ctx.is_node_visible(&id) {
                continue;
            }

            if ctx.graph.is_top_level_group_anchor(id) {
                if let Some(el) = render_group_anchor(
                    ctx,
                    id,
                    &paint_order,
                    &drag_overlay,
                    &intra_by_parent,
                    &selected,
                    stroke,
                    stroke_sel,
                    &mut covered,
                ) {
                    body_children.push(el);
                }
                covered.insert(id);
                for d in ctx.graph.descendants(id) {
                    covered.insert(d);
                }
                continue;
            }

            if ctx.graph.children_of(id).is_empty() {
                body_children.push(render_node_cards(ctx, &[id], "graph-root-leaf"));
                covered.insert(id);
            }
        }

        let cross_layer = if cross_edges.is_empty() {
            None
        } else {
            Some(div().absolute().size_full().child(edges_canvas_element(
                cross_edges,
                selected.clone(),
                stroke,
                stroke_sel,
            )))
        };

        Some(
            div()
                .id("graph-layer")
                .absolute()
                .size_full()
                .child(div().absolute().size_full().children(body_children))
                .children(cross_layer)
                .into_any(),
        )
    }
}

/// Flat canvas: all nodes in [`Graph::paint_order`], all visible edges on top (like legacy plugins).
fn render_flat_graph(
    ctx: &mut RenderContext,
    drag_overlay: &HashSet<crate::NodeId>,
) -> Option<AnyElement> {
    let paint_order = ctx.graph.paint_order();
    let node_ids: Vec<_> = paint_order
        .iter()
        .filter(|id| ctx.is_node_visible(id))
        .filter(|id| !drag_overlay.contains(id))
        .copied()
        .collect();

    let stroke = ctx.theme.edge_stroke;
    let stroke_sel = ctx.theme.edge_stroke_selected;
    let selected = ctx.graph.selected_edge().clone();

    let mut edges: Vec<(EdgeId, Option<EdgeGeometry>)> = Vec::new();
    for (_, edge) in ctx.graph.edges().iter() {
        if !plan::edge_is_visible(ctx, edge) {
            continue;
        }
        ctx.cache_port_offset_with_edge(&edge.id);
        edges.push((edge.id, edge_geometry2(edge, ctx)));
    }

    if node_ids.is_empty() && edges.is_empty() {
        return None;
    }

    let mut layer = div().id("graph-layer").absolute().size_full();
    if !node_ids.is_empty() {
        layer = layer.child(render_node_cards(ctx, &node_ids, "graph-flat-nodes"));
    }
    if !edges.is_empty() {
        layer = layer.child(
            div()
                .absolute()
                .size_full()
                .child(edges_canvas_element(edges, selected, stroke, stroke_sel)),
        );
    }
    Some(layer.into_any())
}

fn render_group_anchor(
    ctx: &mut RenderContext,
    anchor: crate::NodeId,
    paint_order: &[crate::NodeId],
    drag_overlay: &HashSet<crate::NodeId>,
    intra_by_parent: &HashMap<crate::NodeId, Vec<(EdgeId, Option<EdgeGeometry>)>>,
    selected: &HashSet<EdgeId>,
    stroke: u32,
    stroke_sel: u32,
    covered: &mut HashSet<crate::NodeId>,
) -> Option<AnyElement> {
    if drag_overlay.contains(&anchor) {
        return None;
    }

    let mut group = div().id(ElementId::Uuid(*anchor.as_uuid()));

    if let Some(shell) = render_node_shell(ctx, &anchor) {
        group = group.child(shell);
    }

    if let Some(edges) = intra_by_parent.get(&anchor) {
        if !edges.is_empty() {
            group = group.child(
                div()
                    .id(ElementId::Uuid(*anchor.as_uuid()))
                    .absolute()
                    .size_full()
                    .child(edges_canvas_element(
                        edges.clone(),
                        selected.clone(),
                        stroke,
                        stroke_sel,
                    )),
            );
        }
    }

    let child_ids: Vec<_> = paint_order
        .iter()
        .filter(|id| {
            ctx.graph
                .get_node(id)
                .and_then(|n| n.parent())
                .is_some_and(|p| p == anchor)
        })
        .copied()
        .filter(|id| !drag_overlay.contains(id))
        .filter(|id| ctx.is_node_visible(id))
        .collect();

    for child in child_ids {
        covered.insert(child);
        if !ctx.graph.children_of(child).is_empty() {
            if let Some(nested) = render_group_anchor(
                ctx,
                child,
                paint_order,
                drag_overlay,
                intra_by_parent,
                selected,
                stroke,
                stroke_sel,
                covered,
            ) {
                group = group.child(nested);
            }
        } else {
            group = group.child(render_node_cards(ctx, &[child], "graph-child"));
        }
    }

    if let Some(ports) = render_node_ports(ctx, &anchor) {
        group = group.child(ports);
    }

    Some(group.into_any())
}
