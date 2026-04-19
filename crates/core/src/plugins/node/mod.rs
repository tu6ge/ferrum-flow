mod command;
mod drag_events;
mod interaction;

pub use command::DragNodesCommand;
pub use drag_events::{ActiveNodeDrag, NODE_DRAG_TICK_INTERVAL, NodeDragEvent};
use gpui::{Element, ParentElement, div};
pub use interaction::NodeInteractionPlugin;

/// Renders the given nodes (and their ports) like [`NodePlugin`], for use on the interaction overlay.
pub(super) fn render_node_cards(
    ctx: &mut RenderContext,
    node_ids: &[crate::NodeId],
) -> gpui::AnyElement {
    ctx.cache_port_offset_with_nodes(&node_ids.to_vec());
    let list = node_ids.iter().filter_map(|node_id| {
        let node = ctx.graph.nodes().get(node_id)?;
        let render = ctx.renderers.get(node.renderer_key());

        let node_render = render.render(node, ctx);

        let ports = ctx
            .graph
            .ports()
            .iter()
            .filter(|(_, port)| port.node_id() == *node_id)
            .filter_map(|(_, port)| render.port_render(node, port, ctx));

        Some(div().child(node_render).children(ports))
    });

    div().children(list).into_any()
}

use crate::plugin::{Plugin, RenderContext};

pub struct NodePlugin {}

impl NodePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for NodePlugin {
    fn name(&self) -> &'static str {
        "node"
    }
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        _event: &crate::plugin::FlowEvent,
        _context: &mut crate::plugin::PluginContext,
    ) -> crate::plugin::EventResult {
        crate::plugin::EventResult::Continue
    }
    fn priority(&self) -> i32 {
        60
    }
    fn render_layer(&self) -> crate::plugin::RenderLayer {
        crate::plugin::RenderLayer::Nodes
    }
    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let node_ids: Vec<_> = ctx
            .graph
            .node_order()
            .iter()
            .filter(|node_id| {
                ctx.is_node_visible(node_id)
                    && !ctx
                        .get_shared_state::<ActiveNodeDrag>()
                        .is_some_and(|d| d.0.contains(node_id))
            })
            .cloned()
            .collect();

        Some(render_node_cards(ctx, &node_ids))
    }
}
