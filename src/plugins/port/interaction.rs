use gpui::{Element, Pixels, Point, canvas, rgb};

use crate::{
    NodeId, PortId,
    canvas::InteractionHandler,
    plugin::{FlowEvent, InputEvent, Plugin},
    plugins::port::{edge_bezier, port_screen_bounds, port_screen_position},
};

use super::command::CreateEdge;

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
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::plugin::EventResult {
        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event {
            let mouse_world = ctx.viewport.screen_to_world(ev.position);
            if let Some((node_id, port_id)) = ctx
                .graph
                .ports
                .iter()
                .find(|(id, _)| match port_screen_bounds(**id, ctx) {
                    Some(b) => b.contains(&mouse_world),
                    None => false,
                })
                .map(|(_, p)| (p.node_id, p.id))
            {
                ctx.start_interaction(PortConnecting {
                    node_id,
                    port_id,
                    mouse: mouse_world,
                    moving: false,
                });
                return crate::plugin::EventResult::Stop;
            }
        }

        crate::plugin::EventResult::Continue
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

struct PortConnecting {
    node_id: NodeId,
    port_id: PortId,
    moving: bool,
    mouse: Point<Pixels>,
}

impl PortConnecting {}

impl InteractionHandler for PortConnecting {
    fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::canvas::InteractionResult {
        // let mouse_world = ctx.viewport.world_to_screen(event.position);
        self.mouse = event.position;
        self.moving = true;
        ctx.notify();
        crate::canvas::InteractionResult::Continue
    }
    fn on_mouse_up(
        &mut self,
        ev: &gpui::MouseUpEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::canvas::InteractionResult {
        let mouse_world = ctx.viewport.screen_to_world(ev.position);
        if let Some((node_id, port_id)) = ctx
            .graph
            .ports
            .iter()
            .find(|(id, _)| match port_screen_bounds(**id, ctx) {
                Some(b) => b.contains(&mouse_world),
                None => false,
            })
            .map(|(_, p)| (p.node_id, p.id))
        {
            if node_id == self.node_id {
                ctx.cancel_interaction();
                ctx.notify();
                return crate::canvas::InteractionResult::End;
            }
            let connecting_port = &ctx.graph.ports[&self.port_id];
            let target_port = &ctx.graph.ports[&port_id];
            if connecting_port.kind == target_port.kind {
                ctx.cancel_interaction();
                ctx.notify();
                return crate::canvas::InteractionResult::End;
            }
            let edge = ctx
                .graph
                .new_edge()
                .source(self.port_id.clone())
                .target(port_id);

            ctx.execute_command(CreateEdge::new(edge));
        }
        ctx.cancel_interaction();
        crate::canvas::InteractionResult::End
    }
    fn render(&self, ctx: &mut crate::RenderContext) -> Option<gpui::AnyElement> {
        if !self.moving {
            return None;
        }

        let mouse: Point<Pixels> = self.mouse;
        let start = port_screen_position(self.port_id, &ctx)?;
        Some(
            canvas(
                |_, _, _| {},
                move |_, _, win, _| {
                    if let Ok(line) = edge_bezier(start, mouse) {
                        win.paint_path(line, rgb(0xb1b1b8));
                    }
                },
            )
            .into_any(),
        )
    }
}
