//! Meili port drag interaction: matches `ferrum_flow::PortInteractionPlugin`, but clicking the dangling blue
//! endpoint emits [`PickNodeTypeForPendingLink`](super::pick_link_event::PickNodeTypeForPendingLink) for
//! [`super::node_type_picker::NodeTypePickerPlugin`].
//!
//! **Maintenance**: forked from core `plugins/port/interaction.rs`; re-diff when upgrading core. If core later
//! supports “pick type then create node”, switch back to `PortInteractionPlugin` and remove this module.

use ferrum_flow::{
    CreateEdge, FlowEvent, InputEvent, Interaction, Plugin, PortId, PortKind, PortPosition,
    RenderContext, Viewport, edge_bezier, filled_disc_path, port_screen_big_bounds,
    port_screen_bounds,
};
use gpui::{Element, Pixels, Point, canvas, px, rgb};

use super::pick_link_event::PickNodeTypeForPendingLink;

#[derive(Clone, Copy)]
struct PendingPortLink {
    source_port: PortId,
    end_world: Point<Pixels>,
}

#[derive(Clone, Copy)]
struct PendingLinkCommitted {
    source_port: PortId,
    end_world: Point<Pixels>,
}

pub struct MeiliPortInteractionPlugin {
    pending: Option<PendingPortLink>,
}

impl MeiliPortInteractionPlugin {
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
        ctx: &ferrum_flow::PluginContext,
        end_world: Point<Pixels>,
        screen: Point<Pixels>,
    ) -> bool {
        let c = ctx.world_to_screen(end_world);
        let dx: f32 = (screen.x - c.x).into();
        let dy: f32 = (screen.y - c.y).into();
        let rf: f32 = px(10.0).into();
        dx * dx + dy * dy <= rf * rf
    }

    fn paint_wire_and_dot(
        win: &mut gpui::Window,
        start: Point<Pixels>,
        end: Point<Pixels>,
        start_position: PortPosition,
        target_position: PortPosition,
        viewport: &Viewport,
        line_rgb: u32,
        dot_rgb: u32,
    ) {
        if let Ok(path) = edge_bezier(start, start_position, target_position, end, viewport) {
            win.paint_path(path, rgb(line_rgb));
        }
        if let Ok(dot) = filled_disc_path(end, px(6.0)) {
            win.paint_path(dot, rgb(dot_rgb));
        }
    }
}

impl Plugin for MeiliPortInteractionPlugin {
    fn name(&self) -> &'static str {
        "meili_port_interaction"
    }

    fn setup(&mut self, _ctx: &mut ferrum_flow::InitPluginContext) {}

    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut ferrum_flow::PluginContext,
    ) -> ferrum_flow::EventResult {
        if let Some(p) = event.as_custom::<PendingLinkCommitted>() {
            self.pending = Some(PendingPortLink {
                source_port: p.source_port,
                end_world: p.end_world,
            });
            return ferrum_flow::EventResult::Stop;
        }

        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event {
            if let Some(pend) = self.pending {
                if Self::pending_dot_contains_screen(ctx, pend.end_world, ev.position) {
                    self.pending = None;
                    ctx.emit(FlowEvent::custom(PickNodeTypeForPendingLink {
                        source_port: pend.source_port,
                        end_world: pend.end_world,
                    }));
                    return ferrum_flow::EventResult::Stop;
                }
            }

            let mouse_world = ctx.screen_to_world(ev.position);
            let port_hit = ctx
                .graph
                .ports()
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
                return ferrum_flow::EventResult::Stop;
            }

            if self.pending.take().is_some() {
                ctx.notify();
            }
        }

        ferrum_flow::EventResult::Continue
    }

    fn priority(&self) -> i32 {
        125
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let p = self.pending.as_ref()?;
        let port_meta = ctx.graph.get_port(&p.source_port)?;
        let node = ctx.nodes().get(&port_meta.node_id)?;
        let start = ctx.port_screen_center(node, p.source_port)?;
        let end = ctx.world_to_screen(p.end_world);
        let source_port = ctx.graph.get_port(&p.source_port)?;
        let start_position = source_port.position;
        let target_position = Self::facing_position(start_position);
        let viewport = ctx.viewport().clone();
        let line_rgb = ctx.theme.port_preview_line;
        let dot_rgb = ctx.theme.port_preview_dot;

        Some(
            canvas(
                move |_, _, _| (start_position, target_position, viewport, line_rgb, dot_rgb),
                move |_, (sp, tp, vp, lr, dr), win, _| {
                    Self::paint_wire_and_dot(win, start, end, sp, tp, &vp, lr, dr);
                },
            )
            .into_any(),
        )
    }

    fn render_layer(&self) -> ferrum_flow::RenderLayer {
        ferrum_flow::RenderLayer::Interaction
    }
}

struct PortConnecting {
    port_id: PortId,
    position: PortPosition,
    target_position: PortPosition,
    mouse: Option<Point<Pixels>>,
}

impl Interaction for PortConnecting {
    fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        ctx: &mut ferrum_flow::PluginContext,
    ) -> ferrum_flow::InteractionResult {
        self.mouse = Some(event.position);
        let mouse_world = ctx.screen_to_world(event.position);
        if let Some(port) = ctx
            .graph
            .ports()
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
        ferrum_flow::InteractionResult::Continue
    }

    fn on_mouse_up(
        &mut self,
        ev: &gpui::MouseUpEvent,
        ctx: &mut ferrum_flow::PluginContext,
    ) -> ferrum_flow::InteractionResult {
        let mouse_world = ctx.screen_to_world(ev.position);
        if let Some((node_id, port_id)) = ctx
            .graph
            .ports()
            .iter()
            .filter(|(_, port)| ctx.is_node_visible(&port.node_id))
            .find(|(id, _)| match port_screen_bounds(**id, ctx) {
                Some(b) => b.contains(&mouse_world),
                None => false,
            })
            .map(|(_, p)| (p.node_id, p.id))
        {
            let Some(source_node) = ctx.graph.get_port(&self.port_id).map(|p| p.node_id) else {
                return ferrum_flow::InteractionResult::End;
            };
            if node_id == source_node {
                ctx.cancel_interaction();
                ctx.notify();
                return ferrum_flow::InteractionResult::End;
            }
            let Some(connecting_port) = ctx.graph.get_port(&self.port_id) else {
                return ferrum_flow::InteractionResult::End;
            };
            let Some(target_port) = ctx.graph.get_port(&port_id) else {
                return ferrum_flow::InteractionResult::End;
            };
            if connecting_port.kind == target_port.kind {
                ctx.cancel_interaction();
                ctx.notify();
                return ferrum_flow::InteractionResult::End;
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
                    return ferrum_flow::InteractionResult::End;
                }
            };

            ctx.execute_command(CreateEdge::new(edge));
            return ferrum_flow::InteractionResult::End;
        }

        ctx.emit(FlowEvent::custom(PendingLinkCommitted {
            source_port: self.port_id,
            end_world: mouse_world,
        }));
        ferrum_flow::InteractionResult::End
    }

    fn render(&self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let mouse = self.mouse?;
        let port_meta = ctx.graph.get_port(&self.port_id)?;
        let node = ctx.nodes().get(&port_meta.node_id)?;
        let start = ctx.port_screen_center(node, self.port_id)?;
        let position = self.position;
        let target_position = self.target_position;
        let viewport = ctx.viewport().clone();
        let line_rgb = ctx.theme.port_preview_line;
        let dot_rgb = ctx.theme.port_preview_dot;

        Some(
            canvas(
                move |_, _, _| (position, target_position, viewport, line_rgb, dot_rgb),
                move |_, (position, target_position, viewport, lr, dr), win, _| {
                    MeiliPortInteractionPlugin::paint_wire_and_dot(
                        win,
                        start,
                        mouse,
                        position,
                        target_position,
                        &viewport,
                        lr,
                        dr,
                    );
                },
            )
            .into_any(),
        )
    }
}
