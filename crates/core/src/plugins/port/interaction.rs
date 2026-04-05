use gpui::{Element, Pixels, Point, canvas, px, rgb};

use crate::{
    PortId, PortKind, PortPosition,
    canvas::Interaction,
    plugin::{FlowEvent, InputEvent, Plugin, RenderContext},
    plugins::port::{
        edge_bezier, filled_disc_path, port_screen_big_bounds, port_screen_bounds,
        port_screen_position,
    },
};

use super::command::CreateEdge;

/// Dangling link from a port to a world-space endpoint (shown with a dot until the user clicks it).
#[derive(Clone, Copy)]
struct PendingPortLink {
    source_port: PortId,
    end_world: Point<Pixels>,
}

/// Internal: interaction finished on empty canvas — queue for [`PortInteractionPlugin`].
#[derive(Clone, Copy)]
struct PendingLinkCommitted {
    source_port: PortId,
    end_world: Point<Pixels>,
}

pub struct PortInteractionPlugin {
    pending: Option<PendingPortLink>,
}

impl PortInteractionPlugin {
    pub fn new() -> Self {
        Self { pending: None }
    }

    fn facing_position(p: PortPosition) -> PortPosition {
        match p {
            PortPosition::Left => PortPosition::Right,
            PortPosition::Right => PortPosition::Left,
            PortPosition::Top => PortPosition::Bottom,
            PortPosition::Bottom => PortPosition::Top,
        }
    }

    fn pending_dot_contains_screen(
        ctx: &crate::plugin::PluginContext,
        end_world: Point<Pixels>,
        screen: Point<Pixels>,
    ) -> bool {
        let c = ctx.world_to_screen(end_world);
        let dx: f32 = (screen.x - c.x).into();
        let dy: f32 = (screen.y - c.y).into();
        let rf: f32 = px(10.0).into();
        dx * dx + dy * dy <= rf * rf
    }

    fn finish_pending_link(&mut self, ctx: &mut crate::plugin::PluginContext, p: PendingPortLink) {
        let Some(source) = ctx.graph.ports.get(&p.source_port).cloned() else {
            return;
        };

        let x: f32 = p.end_world.x.into();
        let y: f32 = p.end_world.y.into();

        let mut builder = ctx.create_node("");
        builder = builder.position(x, y);
        builder = match source.kind {
            PortKind::Output => builder.input(),
            PortKind::Input => builder.output(),
        };

        let (new_node, new_ports) = builder.only_build(ctx.graph);

        let edge = match source.kind {
            PortKind::Output => {
                let Some(in_port) = new_node.inputs.first().copied() else {
                    return;
                };
                ctx.new_edge().source(p.source_port).target(in_port)
            }
            PortKind::Input => {
                let Some(out_port) = new_node.outputs.first().copied() else {
                    return;
                };
                ctx.new_edge().source(out_port).target(p.source_port)
            }
        };

        ctx.execute_command(super::command::CreateNode::new(new_node));
        for port in new_ports {
            ctx.execute_command(super::command::CreatePort::new(port));
        }

        ctx.execute_command(CreateEdge::new(edge));
    }

    fn paint_wire_and_dot(
        win: &mut gpui::Window,
        start: Point<Pixels>,
        end: Point<Pixels>,
        start_position: PortPosition,
        target_position: PortPosition,
        viewport: &crate::Viewport,
    ) {
        if let Ok(line) = edge_bezier(start, start_position, target_position, end, viewport) {
            win.paint_path(line, rgb(0xb1b1b8));
        }
        if let Ok(dot) = filled_disc_path(end, px(6.0)) {
            win.paint_path(dot, rgb(0x6B9EFF));
        }
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
        if let Some(p) = event.as_custom::<PendingLinkCommitted>() {
            self.pending = Some(PendingPortLink {
                source_port: p.source_port,
                end_world: p.end_world,
            });
            return crate::plugin::EventResult::Stop;
        }

        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event {
            if let Some(pend) = self.pending {
                if Self::pending_dot_contains_screen(ctx, pend.end_world, ev.position) {
                    self.pending = None;
                    self.finish_pending_link(ctx, pend);
                    return crate::plugin::EventResult::Stop;
                }
            }

            let mouse_world = ctx.viewport.screen_to_world(ev.position);
            let port_hit = ctx
                .graph
                .ports
                .iter()
                .filter(|(_, port)| ctx.is_node_visible(&port.node_id))
                .find(|(id, _)| match port_screen_bounds(**id, ctx) {
                    Some(b) => b.contains(&mouse_world),
                    None => false,
                })
                .map(|(_, p)| (p.id, p.position));

            if let Some((port_id, position)) = port_hit {
                self.pending = None;
                ctx.start_interaction(PortConnecting {
                    port_id,
                    position,
                    target_position: PortPosition::Left,
                    mouse: Some(ev.position),
                });
                return crate::plugin::EventResult::Stop;
            }

            if self.pending.take().is_some() {
                ctx.notify();
            }
        }

        crate::plugin::EventResult::Continue
    }
    fn priority(&self) -> i32 {
        125
    }
    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let p = self.pending.as_ref()?;
        let start = port_screen_position(p.source_port, ctx)?;
        let end = ctx.world_to_screen(p.end_world);
        let source_port = ctx.graph.ports.get(&p.source_port)?;
        let start_position = source_port.position;
        let target_position = Self::facing_position(start_position);
        let viewport = ctx.viewport.clone();

        Some(
            canvas(
                move |_, _, _| (start_position, target_position, viewport),
                move |_, (sp, tp, vp), win, _| {
                    Self::paint_wire_and_dot(win, start, end, sp, tp, &vp);
                },
            )
            .into_any(),
        )
    }
    fn render_layer(&self) -> crate::plugin::RenderLayer {
        crate::plugin::RenderLayer::Interaction
    }
}

struct PortConnecting {
    port_id: PortId,
    position: PortPosition,
    target_position: PortPosition,
    /// Cursor in **screen** space (matches port_screen_position / bezier end).
    mouse: Option<Point<Pixels>>,
}

impl Interaction for PortConnecting {
    fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::canvas::InteractionResult {
        self.mouse = Some(event.position);
        let mouse_world = ctx.screen_to_world(event.position);
        if let Some(port) = ctx
            .graph
            .ports
            .iter()
            .filter(|(_, port)| ctx.is_node_visible(&port.node_id))
            .find(|(id, _)| match port_screen_big_bounds(**id, ctx) {
                Some(b) => b.contains(&mouse_world),
                None => false,
            })
            .map(|(_, p)| p)
        {
            if port.id != self.port_id {
                self.target_position = port.position;
            }
        }
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
            let source_node = ctx.graph.ports[&self.port_id].node_id;
            if node_id == source_node {
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
            let edge = match (connecting_port.kind, target_port.kind) {
                (PortKind::Output, PortKind::Input) => {
                    ctx.new_edge().source(self.port_id).target(port_id)
                }
                (PortKind::Input, PortKind::Output) => {
                    ctx.new_edge().source(port_id).target(self.port_id)
                }
                _ => {
                    ctx.cancel_interaction();
                    ctx.notify();
                    return crate::canvas::InteractionResult::End;
                }
            };

            ctx.execute_command(CreateEdge::new(edge));
            return crate::canvas::InteractionResult::End;
        }

        ctx.emit(FlowEvent::custom(PendingLinkCommitted {
            source_port: self.port_id,
            end_world: mouse_world,
        }));
        crate::canvas::InteractionResult::End
    }
    fn render(&self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let mouse = self.mouse?;
        let start = port_screen_position(self.port_id, ctx)?;
        let position = self.position;
        let target_position = self.target_position;
        let viewport = ctx.viewport.clone();

        Some(
            canvas(
                move |_, _, _| (position, target_position, viewport),
                move |_, (position, target_position, viewport), win, _| {
                    PortInteractionPlugin::paint_wire_and_dot(
                        win,
                        start,
                        mouse,
                        position,
                        target_position,
                        &viewport,
                    );
                },
            )
            .into_any(),
        )
    }
}
