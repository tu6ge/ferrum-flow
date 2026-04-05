use gpui::{Pixels, Point, px};

use crate::{
    canvas::{Command, CommandContext},
    plugin::{FlowEvent, Plugin, PluginContext, primary_platform_modifier},
};

/// Pan + zoom the viewport so selected nodes fit the window (⌘⇧F / Ctrl⇧F). Undo restores prior view.
pub struct FocusSelectionPlugin;

impl FocusSelectionPlugin {
    pub fn new() -> Self {
        Self
    }
}

fn focus_shortcut(ev: &gpui::KeyDownEvent) -> bool {
    primary_platform_modifier(ev) && ev.keystroke.modifiers.shift
}

/// Same zoom limits as [`crate::plugins::ViewportPlugin`] wheel zoom.
const ZOOM_MIN: f32 = 0.7;
const ZOOM_MAX: f32 = 3.0;
/// Inset from window edges (ratio per side).
const MARGIN_RATIO: f32 = 0.08;

struct ViewportFrameCommand {
    from_zoom: f32,
    from_offset: Point<Pixels>,
    to_zoom: f32,
    to_offset: Point<Pixels>,
}

impl Command for ViewportFrameCommand {
    fn name(&self) -> &'static str {
        "viewport_frame"
    }

    fn execute(&mut self, ctx: &mut CommandContext) {
        ctx.viewport.zoom = self.to_zoom;
        ctx.viewport.offset.x = self.to_offset.x;
        ctx.viewport.offset.y = self.to_offset.y;
    }

    fn undo(&mut self, ctx: &mut CommandContext) {
        ctx.viewport.zoom = self.from_zoom;
        ctx.viewport.offset.x = self.from_offset.x;
        ctx.viewport.offset.y = self.from_offset.y;
    }

    fn to_ops(&self, _ctx: &mut CommandContext) -> Vec<crate::GraphOp> {
        vec![]
    }
}

fn focus_selected(ctx: &mut PluginContext) {
    let Some(bounds) = ctx.graph.selection_bounds() else {
        return;
    };
    let Some(wb) = ctx.viewport.window_bounds else {
        return;
    };

    let win_w: f32 = wb.size.width.into();
    let win_h: f32 = wb.size.height.into();
    if win_w <= 0.0 || win_h <= 0.0 {
        return;
    }

    let bx: f32 = bounds.origin.x.into();
    let by: f32 = bounds.origin.y.into();
    let bw: f32 = bounds.size.width.into();
    let bh: f32 = bounds.size.height.into();
    let bw_safe = bw.max(1.0);
    let bh_safe = bh.max(1.0);

    let avail_w = win_w * (1.0 - 2.0 * MARGIN_RATIO);
    let avail_h = win_h * (1.0 - 2.0 * MARGIN_RATIO);
    let z = (avail_w / bw_safe)
        .min(avail_h / bh_safe)
        .clamp(ZOOM_MIN, ZOOM_MAX);

    let cx = bx + bw / 2.0;
    let cy = by + bh / 2.0;
    let center_x = win_w / 2.0;
    let center_y = win_h / 2.0;
    let new_offset = Point::new(px(center_x - cx * z), px(center_y - cy * z));

    let from_zoom = ctx.viewport.zoom;
    let from_offset = ctx.viewport.offset;
    let zoom_changed = (from_zoom - z).abs() > 1e-4;
    let ox: f32 = from_offset.x.into();
    let oy: f32 = from_offset.y.into();
    let nx: f32 = new_offset.x.into();
    let ny: f32 = new_offset.y.into();
    let offset_changed = (ox - nx).abs() > 0.5 || (oy - ny).abs() > 0.5;
    if !zoom_changed && !offset_changed {
        return;
    }

    ctx.execute_command(ViewportFrameCommand {
        from_zoom,
        from_offset,
        to_zoom: z,
        to_offset: new_offset,
    });
}

impl Plugin for FocusSelectionPlugin {
    fn name(&self) -> &'static str {
        "focus_selection"
    }

    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}

    fn priority(&self) -> i32 {
        90
    }

    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut PluginContext,
    ) -> crate::plugin::EventResult {
        if let FlowEvent::Input(crate::plugin::InputEvent::KeyDown(ev)) = event {
            if focus_shortcut(ev) && ev.keystroke.key == "f" {
                focus_selected(ctx);
                return crate::plugin::EventResult::Stop;
            }
        }
        crate::plugin::EventResult::Continue
    }
}
