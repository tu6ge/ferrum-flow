use gpui::{Pixels, Point, px};

use crate::{
    canvas::{Command, Interaction, InteractionResult},
    plugin::{EventResult, FlowEvent, InputEvent, Plugin},
};

pub struct ViewportPlugin;

impl ViewportPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for ViewportPlugin {
    fn name(&self) -> &'static str {
        "viewport"
    }
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        event: &crate::plugin::FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> EventResult {
        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event
            && ev.modifiers.shift
        {
            ctx.start_interaction(Panning {
                start_mouse: ev.position,
                start_offset: ctx.viewport.offset,
            });
            return EventResult::Stop;
        } else if let FlowEvent::Input(InputEvent::Wheel(ev)) = event {
            let cursor = ev.position;

            let before = ctx.screen_to_world(cursor);

            let delta = f32::from(ev.delta.pixel_delta(px(1.0)).y);
            if delta == 0.0 {
                return EventResult::Continue;
            }

            let zoom_delta = if delta > 0.0 { 0.9 } else { 1.1 };

            ctx.viewport.zoom *= zoom_delta;

            ctx.viewport.zoom = ctx.viewport.zoom.clamp(0.7, 3.0);

            let after = ctx.world_to_screen(before);

            ctx.viewport.offset.x += cursor.x - after.x;
            ctx.viewport.offset.y += cursor.y - after.y;
            ctx.notify();
        }
        EventResult::Continue
    }
    fn priority(&self) -> i32 {
        10
    }
    fn render(&mut self, _context: &mut crate::plugin::RenderContext) -> Option<gpui::AnyElement> {
        None
    }
}

struct Panning {
    start_mouse: Point<Pixels>,
    start_offset: Point<Pixels>,
}

impl Interaction for Panning {
    fn on_mouse_move(
        &mut self,
        ev: &gpui::MouseMoveEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> InteractionResult {
        let dx = ev.position.x - self.start_mouse.x;
        let dy = ev.position.y - self.start_mouse.y;

        ctx.viewport.offset.x = self.start_offset.x + dx;
        ctx.viewport.offset.y = self.start_offset.y + dy;
        ctx.notify();

        InteractionResult::Continue
    }
    fn on_mouse_up(
        &mut self,
        _event: &gpui::MouseUpEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::canvas::InteractionResult {
        ctx.execute_command(PanningCommand {
            from: self.start_offset,
            to: ctx.viewport.offset,
        });
        ctx.cancel_interaction();
        InteractionResult::End
    }
    fn render(&self, _ctx: &mut crate::plugin::RenderContext) -> Option<gpui::AnyElement> {
        None
    }
}

struct PanningCommand {
    from: Point<Pixels>,
    to: Point<Pixels>,
}

impl Command for PanningCommand {
    fn name(&self) -> &'static str {
        "panning"
    }
    fn execute(&mut self, ctx: &mut crate::canvas::CommandContext) {
        ctx.viewport.offset.x = self.to.x;
        ctx.viewport.offset.y = self.to.y;
    }
    fn undo(&mut self, ctx: &mut crate::canvas::CommandContext) {
        ctx.viewport.offset.x = self.from.x;
        ctx.viewport.offset.y = self.from.y;
    }

    fn to_ops(&self, _ctx: &mut crate::CommandContext) -> Vec<crate::GraphOp> {
        vec![]
    }
}
