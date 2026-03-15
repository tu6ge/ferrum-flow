use crate::plugin::Plugin;

pub struct PortInteractionPlugin;

impl PortInteractionPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for PortInteractionPlugin {
    fn name(&self) -> &'static str {
        "port_interaction"
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
        125
    }
    fn render(&mut self, _context: &mut crate::RenderContext) -> Option<gpui::AnyElement> {
        None
    }
    fn render_layer(&self) -> crate::plugin::RenderLayer {
        crate::plugin::RenderLayer::Interaction
    }
}
