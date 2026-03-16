mod command;
mod interaction;
mod renderer;

pub use renderer::{NodeRenderer, RendererRegistry};

use gpui::{Element, ParentElement, div};
pub use interaction::NodeInteractionPlugin;

use crate::plugin::Plugin;

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
    fn render(&mut self, ctx: &mut crate::plugin::RenderContext) -> Option<gpui::AnyElement> {
        let list: Vec<_> = ctx
            .graph
            .node_order()
            .iter()
            .filter_map(|node_id| {
                let node = ctx.graph.nodes().get(node_id)?;
                let render = self.renderers.get(&node.node_type);
                Some(render.render(node, ctx))
            })
            .collect();

        Some(div().children(list).into_any())
    }
}
