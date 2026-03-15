use gpui::{Bounds, Element, Path, PathBuilder, Pixels, Point, Size, canvas, px, rgb};

use crate::{
    Node, NodeId, Port, PortId, PortKind, RenderContext,
    canvas::InteractionHandler,
    plugin::{FlowEvent, InputEvent, Plugin},
};

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
                .find(|(id, _)| port_screen_bounds(**id, ctx).contains(&mouse_world))
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
            .find(|(id, _)| port_screen_bounds(**id, ctx).contains(&mouse_world))
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

            ctx.graph.add_edge(edge);
            ctx.cancel_interaction();
            ctx.notify();
        }
        crate::canvas::InteractionResult::End
    }
    fn render(&self, ctx: &mut crate::RenderContext) -> Option<gpui::AnyElement> {
        if !self.moving {
            return None;
        }

        let mouse: Point<Pixels> = self.mouse;
        let start = port_screen_position(self.port_id, &ctx);
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

fn port_offset(node: &Node, port: &Port) -> Point<Pixels> {
    let node_size = node.size;

    match port.kind {
        PortKind::Input => Point::new(px(0.0), node_size.height / 2.0),

        PortKind::Output => Point::new(node_size.width, node_size.height / 2.0),
    }
}

fn port_screen_bounds(port_id: PortId, ctx: &crate::plugin::PluginContext) -> Bounds<Pixels> {
    let port = &ctx.graph.ports[&port_id];
    let node = &ctx.graph.nodes()[&port.node_id];

    let node_pos = node.point();

    let offset = port_offset(node, port);

    Bounds::new(
        node_pos + offset - Point::new(px(6.0), px(6.0)),
        Size::new(px(12.0), px(12.0)),
    )
}
fn port_screen_position(port_id: PortId, ctx: &RenderContext) -> Point<Pixels> {
    let port = &ctx.graph.ports[&port_id];
    let node = &ctx.graph.nodes()[&port.node_id];

    let node_pos = node.point();

    let offset = port_offset(node, port);

    ctx.viewport.world_to_screen(node_pos + offset)
}

fn edge_bezier(start: Point<Pixels>, end: Point<Pixels>) -> Result<Path<Pixels>, anyhow::Error> {
    let mut line = PathBuilder::stroke(px(1.0));
    line.move_to(start);
    line.cubic_bezier_to(
        end,
        Point::new(start.x + px(50.0), start.y),
        Point::new(end.x - px(50.0), end.y),
    );

    line.build()
}
