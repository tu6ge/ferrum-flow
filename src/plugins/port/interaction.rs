use gpui::{Element, Pixels, Point, canvas, rgb};

use crate::{
    NodeId, PortId, PortPosition,
    canvas::Interaction,
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
        // ctx.cache_all_node_port_offset();

        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event {
            let mouse_world = ctx.viewport.screen_to_world(ev.position);
            if let Some((node_id, port_id, position)) = ctx
                .graph
                .ports
                .iter()
                .filter(|(_, port)| ctx.is_node_visible(&port.node_id))
                .find(|(id, _)| match port_screen_bounds(**id, ctx) {
                    Some(b) => b.contains(&mouse_world),
                    None => false,
                })
                .map(|(_, p)| (p.node_id, p.id, p.position))
            {
                ctx.start_interaction(PortConnecting {
                    node_id,
                    port_id,
                    position,
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
    position: PortPosition,
    moving: bool,
    mouse: Point<Pixels>,
}

impl PortConnecting {}

impl Interaction for PortConnecting {
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
        let mouse_world = ctx.screen_to_world(ev.position);
        if let Some((node_id, port_id)) = ctx
            .graph
            .ports
            .iter()
            .filter(|(_, port)| ctx.is_node_visible(&port.node_id))
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
            let edge = ctx.new_edge().source(self.port_id.clone()).target(port_id);

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
        let position = self.position;
        Some(
            canvas(
                move |_, _, _| position,
                move |_, position, win, _| {
                    if let Ok(line) = edge_bezier(start, position, mouse) {
                        win.paint_path(line, rgb(0xb1b1b8));
                    }
                },
            )
            .into_any(),
        )
    }
}
