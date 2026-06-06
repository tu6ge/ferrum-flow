use crate::viewport_frame::frame_world_rect;
use ferrum_flow_core::{
    EventResult, FlowEvent, InputEvent, Plugin, PluginContext, primary_platform_modifier,
};

/// Pan + zoom the viewport so selected nodes fit the window (⌘⇧F / Ctrl⇧F). Undo restores prior view.
pub struct FocusSelectionPlugin;

impl FocusSelectionPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FocusSelectionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn focus_shortcut(ev: &gpui::KeyDownEvent) -> bool {
    primary_platform_modifier(ev) && ev.keystroke.modifiers.shift
}

fn focus_selected(ctx: &mut PluginContext) {
    let Some(bounds) = ctx.graph.selection_bounds() else {
        return;
    };
    let bx: f32 = bounds.origin.x.into();
    let by: f32 = bounds.origin.y.into();
    let bw: f32 = bounds.size.width.into();
    let bh: f32 = bounds.size.height.into();
    frame_world_rect(ctx, bx, by, bw, bh);
}

pub(crate) fn focus_viewport_on_selection(ctx: &mut PluginContext) {
    focus_selected(ctx);
}

impl Plugin for FocusSelectionPlugin {
    fn name(&self) -> &'static str {
        "focus_selection"
    }

    fn priority(&self) -> i32 {
        90
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if let FlowEvent::Input(InputEvent::KeyDown(ev)) = event
            && focus_shortcut(ev)
            && ev.keystroke.key == "f"
        {
            focus_selected(ctx);
            return EventResult::Stop;
        }
        EventResult::Continue
    }
}
