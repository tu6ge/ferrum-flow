use ferrum_flow_core::{
    EventResult, FlowEvent, InputEvent, Plugin, PluginContext, primary_platform_modifier,
};

pub struct HistoryPlugin;

impl HistoryPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for HistoryPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for HistoryPlugin {
    fn name(&self) -> &'static str {
        "history"
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if let FlowEvent::Input(InputEvent::KeyDown(ev)) = event {
            let primary = primary_platform_modifier(ev);
            if ev.keystroke.key == "z" && primary && ev.keystroke.modifiers.shift {
                ctx.redo();
                return EventResult::Stop;
            } else if ev.keystroke.key == "z" && primary {
                ctx.undo();
                return EventResult::Stop;
            }
        }
        EventResult::Continue
    }
}
