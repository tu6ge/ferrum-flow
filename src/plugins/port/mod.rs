use crate::plugin::Plugin;

mod interaction;

pub use interaction::PortInteractionPlugin;

pub struct PortPlugin;

impl PortPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for PortPlugin {
    fn name(&self) -> &'static str {
        "port"
    }
    fn setup(&mut self, ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        _event: &crate::plugin::FlowEvent,
        _context: &mut crate::plugin::PluginContext,
    ) -> crate::plugin::EventResult {
        todo!()
    }
    fn priority(&self) -> i32 {
        70
    }
    fn render_layer(&self) -> crate::plugin::RenderLayer {
        crate::plugin::RenderLayer::Nodes
    }
}
