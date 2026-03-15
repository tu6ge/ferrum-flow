use crate::plugin::{FlowEvent, Plugin};

pub struct DeletePlugin;

impl DeletePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for DeletePlugin {
    fn name(&self) -> &'static str {
        "delete"
    }
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::plugin::EventResult {
        if let FlowEvent::Input(crate::plugin::InputEvent::KeyDown(ev)) = event {
            if ev.keystroke.key == "delete" || ev.keystroke.key == "backspace" {
                if ctx.graph.remove_selected_edge() {
                    ctx.notify();
                }
                if ctx.graph.remove_selected_node() {
                    ctx.notify();
                }
                return crate::plugin::EventResult::Stop;
            }
        }
        crate::plugin::EventResult::Continue
    }
}
