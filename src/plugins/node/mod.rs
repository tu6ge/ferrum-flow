mod command;
mod interaction;
mod renderer;

pub use command::DragNodesCommand;
use gpui::{Element, ParentElement, Styled as _, div, px, rgb};
pub use interaction::NodeInteractionPlugin;

use crate::{NodeId, RenderContext, plugin::Plugin, plugins::port::port_screen_position};

pub struct NodePlugin {}

impl NodePlugin {
    pub fn new() -> Self {
        Self {}
    }

    fn render_ports(&self, node_id: &NodeId, ctx: &RenderContext) -> Option<gpui::AnyElement> {
        let ports = ctx
            .graph
            .ports
            .iter()
            .filter(|(_, port)| port.node_id == *node_id)
            .filter_map(|(id, port)| {
                let size = port.size;
                let position = port_screen_position(*id, &ctx)?;

                Some(
                    div()
                        .absolute()
                        .left(position.x - size.width / 2.0 * ctx.viewport.zoom)
                        .top(position.y - size.height / 2.0 * ctx.viewport.zoom)
                        .w(size.width * ctx.viewport.zoom)
                        .h(size.height * ctx.viewport.zoom)
                        .rounded_full()
                        .bg(rgb(0x1A192B)),
                )
            });

        Some(div().children(ports).into_any())
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
            .filter(|node_id| ctx.is_node_visible(node_id))
            .cloned()
            .collect();

        ctx.cache_port_offset_with_nodes(&node_ids);

        let list = node_ids.iter().filter_map(|node_id| {
            let node = ctx.graph.nodes().get(node_id)?;
            let render = ctx.renderers.get(&node.node_type);

            match self.render_ports(node_id, &ctx) {
                Some(ports) => Some(
                    div()
                        .child(render.render(node, ctx))
                        .child(ports)
                        .into_any(),
                ),
                None => Some(render.render(node, ctx)),
            }
        });

        Some(div().children(list).into_any())
    }
}
