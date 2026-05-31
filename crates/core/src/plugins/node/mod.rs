mod command;
mod drag_events;
mod interaction;

use std::collections::HashSet;

use crate::Graph;
pub use command::DragNodesCommand;
pub use drag_events::{ActiveNodeDrag, NODE_DRAG_TICK_INTERVAL, NodeDragEvent};
use gpui::{Element as _, ElementId, InteractiveElement as _, ParentElement, Styled as _, div};
pub use interaction::{BoundaryDragPolicy, NodeInteractionPlugin};

/// Renders the given nodes (and their ports) like [`NodePlugin`], for use on the interaction overlay.
pub(super) fn render_node_cards(
    ctx: &mut RenderContext,
    node_ids: &[crate::NodeId],
    id: &'static str,
) -> gpui::AnyElement {
    ctx.cache_port_offset_with_nodes(node_ids);
    let list = node_ids.iter().filter_map(|node_id| {
        let node = ctx.graph.nodes().get(node_id)?;
        let render = ctx.renderers.get(node.renderer_key());

        let node_render = render.render(node, ctx);

        let port_ids: Vec<crate::PortId> = ctx.cached_port_ids_for_node(node_id).collect();
        let ports = port_ids.iter().filter_map(|port_id| {
            let port = ctx.graph.get_port(port_id)?;
            render.port_render(node, port, ctx)
        });

        Some(
            div()
                .id(ElementId::Uuid(*node_id.as_uuid()))
                .child(node_render)
                .children(ports),
        )
    });

    div().id(id).children(list).into_any()
}

/// Parent group background only (ports rendered separately for z-order).
pub(super) fn render_node_shell(
    ctx: &mut RenderContext,
    node_id: &crate::NodeId,
) -> Option<gpui::AnyElement> {
    let node = ctx.graph.nodes().get(node_id)?;
    let render = ctx.renderers.get(node.renderer_key());
    ctx.cache_port_offset_with_nodes(&[*node_id]);
    Some(
        div()
            .id(ElementId::Uuid(*node_id.as_uuid()))
            .child(render.render(node, ctx))
            .into_any(),
    )
}

/// Ports for a node after its shell and children exist (intra-parent edges stay underneath).
pub(super) fn render_node_ports(
    ctx: &mut RenderContext,
    node_id: &crate::NodeId,
) -> Option<gpui::AnyElement> {
    let node = ctx.graph.nodes().get(node_id)?;
    let render = ctx.renderers.get(node.renderer_key());
    let port_ids: Vec<crate::PortId> = ctx.cached_port_ids_for_node(node_id).collect();
    let ports = port_ids.iter().filter_map(|port_id| {
        let port = ctx.graph.get_port(port_id)?;
        render.port_render(node, port, ctx)
    });
    Some(
        div()
            .id(ElementId::Uuid(uuid::Uuid::new_v4()))
            .absolute()
            .size_full()
            .children(ports)
            .into_any(),
    )
}

/// Drag overlay + static-layer exclusion: dragged roots and all descendants, in [`Graph::paint_order`].
pub(super) fn node_ids_for_drag_overlay(graph: &Graph, dragged: &[NodeId]) -> Vec<NodeId> {
    let mut in_subtree = HashSet::new();
    for &id in dragged {
        in_subtree.insert(id);
        for child in graph.descendants(id) {
            in_subtree.insert(child);
        }
    }
    graph
        .paint_order()
        .into_iter()
        .filter(|id| in_subtree.contains(id))
        .collect()
}

use std::sync::Arc;

use crate::NodeId;
use crate::plugin::{Plugin, RenderContext};
use crate::viewport::ViewportVisibilityCacheKey;

/// Invalidates [`NodePlugin::static_layer_node_ids`] when the viewport changes **or** the active
/// node-drag overlay set changes ([`ActiveNodeDrag`] `Arc` identity + length).
#[derive(Clone, Copy, Debug, PartialEq)]
struct NodeStaticLayerCacheKey {
    viewport: ViewportVisibilityCacheKey,
    nodes_len: usize,
    paint_order_len: usize,
    paint_order_tail: Option<u128>,
    /// `None` when not dragging; else [`Arc::as_ptr`] + len of the shared drag id list.
    drag_arc: Option<(usize, usize)>,
}

impl NodeStaticLayerCacheKey {
    fn from_render_ctx(ctx: &RenderContext) -> Self {
        let drag = ctx.get_shared_state::<ActiveNodeDrag>();
        let paint_order = ctx.graph.paint_order();
        Self {
            viewport: ctx.viewport().visibility_cache_key(),
            nodes_len: ctx.graph.nodes().len(),
            paint_order_len: paint_order.len(),
            paint_order_tail: paint_order.last().map(|id| id.as_uuid().as_u128()),
            drag_arc: drag.map(|d| {
                let p = Arc::as_ptr(&d.0);
                (p.cast::<NodeId>() as usize, d.0.len())
            }),
        }
    }
}

pub struct NodePlugin {
    static_layer_cache_key: Option<NodeStaticLayerCacheKey>,
    /// Viewport-visible nodes for the static [`RenderLayer::Nodes`] layer, excluding the active
    /// drag subtree ([`node_ids_for_drag_overlay`]) rendered on the interaction overlay.
    static_layer_node_ids: Vec<NodeId>,
}

impl NodePlugin {
    pub fn new() -> Self {
        Self {
            static_layer_cache_key: None,
            static_layer_node_ids: Vec::new(),
        }
    }
}

impl Default for NodePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for NodePlugin {
    fn name(&self) -> &'static str {
        "node"
    }
    fn priority(&self) -> i32 {
        60
    }
    fn render_layer(&self) -> crate::plugin::RenderLayer {
        crate::plugin::RenderLayer::Nodes
    }
    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let key = NodeStaticLayerCacheKey::from_render_ctx(ctx);
        if self.static_layer_cache_key != Some(key) {
            self.static_layer_cache_key = Some(key);
            let active = ctx.get_shared_state::<ActiveNodeDrag>();
            let drag_overlay = active
                .map(|d| node_ids_for_drag_overlay(ctx.graph, d.0.as_ref()))
                .unwrap_or_default();
            let drag_overlay: HashSet<NodeId> = drag_overlay.into_iter().collect();
            self.static_layer_node_ids = ctx
                .graph
                .paint_order()
                .iter()
                .filter(|node_id| ctx.is_node_visible(node_id))
                .filter(|node_id| !drag_overlay.contains(node_id))
                .copied()
                .collect();
        }

        if self.static_layer_node_ids.is_empty() {
            return None;
        }

        Some(render_node_cards(
            ctx,
            &self.static_layer_node_ids,
            "static-layer-node-cards",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::node_ids_for_drag_overlay;
    use crate::Graph;

    #[test]
    fn drag_overlay_includes_descendants_in_paint_order() {
        let mut g = Graph::new();
        let a = g.create_node("default").build();
        let b = g.create_node("default").build();
        let c = g.create_node("default").build();
        g.add_child(a, b).unwrap();
        g.add_child(b, c).unwrap();

        assert_eq!(node_ids_for_drag_overlay(&g, &[a]), vec![a, b, c]);
        assert_eq!(node_ids_for_drag_overlay(&g, &[b]), vec![b, c]);
        assert_eq!(node_ids_for_drag_overlay(&g, &[c]), vec![c]);
    }
}
