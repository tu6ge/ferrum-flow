mod command;
mod interaction;
mod renderer;

pub use renderer::{NodeRenderer, RendererRegistry};

use gpui::{Element, ParentElement, Styled as _, div, px, rgb};
pub use interaction::NodeInteractionPlugin;

use crate::{NodeId, RenderContext, plugin::Plugin, plugins::port::port_screen_position};

pub struct NodePlugin {
    renderers: RendererRegistry,
}

impl NodePlugin {
    pub fn new() -> Self {
        Self {
            renderers: RendererRegistry::new(),
        }
    }
    pub fn register_node<R>(mut self, name: impl Into<String>, renderer: R) -> Self
    where
        R: NodeRenderer + 'static,
    {
        self.renderers.register(name, renderer);
        self
    }

    fn render_ports(&self, node_id: &NodeId, ctx: &RenderContext) -> Option<gpui::AnyElement> {
        let ports: Vec<_> = ctx
            .graph
            .ports
            .iter()
            .filter(|(_, port)| port.node_id == *node_id)
            .filter_map(|(id, _)| {
                let position = port_screen_position(*id, &ctx)?;

                Some(
                    div()
                        .absolute()
                        .left(position.x - px(6.0 * ctx.viewport.zoom))
                        .top(position.y - px(6.0 * ctx.viewport.zoom))
                        .w(px(12.0 * ctx.viewport.zoom))
                        .h(px(12.0 * ctx.viewport.zoom))
                        .rounded_full()
                        .bg(rgb(0x1A192B)),
                )
            })
            .collect();

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
        let list: Vec<_> = ctx
            .graph
            .node_order()
            .iter()
            .filter_map(|node_id| {
                let node = ctx.graph.nodes().get(node_id)?;
                let render = self.renderers.get(&node.node_type);

                match self.render_ports(node_id, &ctx) {
                    Some(ports) => Some(
                        div()
                            .child(render.render(node, ctx))
                            .child(ports)
                            .into_any(),
                    ),
                    None => Some(render.render(node, ctx)),
                }
            })
            .collect();

        Some(div().children(list).into_any())
    }
}
