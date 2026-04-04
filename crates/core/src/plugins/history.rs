use crate::plugin::{FlowEvent, Plugin};

pub struct HistoryPlugin;

impl HistoryPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for HistoryPlugin {
    fn name(&self) -> &'static str {
        "history"
    }
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::plugin::EventResult {
        if let FlowEvent::Input(crate::plugin::InputEvent::KeyDown(ev)) = event {
            #[cfg(target_os = "macos")]
            let platform = ev.keystroke.modifiers.platform;

            #[cfg(not(target_os = "macos"))]
            let platform = ev.keystroke.modifiers.control;

            if ev.keystroke.key == "z" && platform && ev.keystroke.modifiers.shift {
                ctx.redo();
                return crate::plugin::EventResult::Stop;
            } else if ev.keystroke.key == "z" && platform {
                ctx.undo();
                return crate::plugin::EventResult::Stop;
            }
        }
        crate::plugin::EventResult::Continue
    }
}
