use crate::{
    canvas::CanvasState,
    plugin::{FlowEvent, Plugin},
};

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
            if ev.keystroke.key == "z"
                && ev.keystroke.modifiers.platform
                && ev.keystroke.modifiers.shift
            {
                let mut state = CanvasState {
                    graph: &mut ctx.graph,
                    viewport: &mut ctx.viewport,
                };
                ctx.history.redo(&mut state);
                return crate::plugin::EventResult::Stop;
            } else if ev.keystroke.key == "z" && ev.keystroke.modifiers.platform {
                let mut state = CanvasState {
                    graph: &mut ctx.graph,
                    viewport: &mut ctx.viewport,
                };
                ctx.history.undo(&mut state);
                return crate::plugin::EventResult::Stop;
            }
        }
        crate::plugin::EventResult::Continue
    }
}
