//! # Node type picker after a dangling link (Meili)
//!
//! ## Flow
//!
//! 1. Drag a wire from a port and release on empty canvas → "dangling link" state (gray wire + blue endpoint).
//! 2. Click the **blue endpoint** → [`MeiliPortInteractionPlugin`](super::meili_port_interaction::MeiliPortInteractionPlugin)
//!    emits [`PickNodeTypeForPendingLink`](super::pick_link_event::PickNodeTypeForPendingLink); this plugin stores it
//!    in [`crate::pick_state`] and notifies the canvas.
//! 3. [`crate::shell::MeiliShell`] sees `pick_state` and renders the **gpui-component**
//!    [`Select`](gpui_component::select::Select); after the user picks, the Shell calls [`ferrum_flow::FlowCanvas::dispatch_command`]
//!    with [`NodeTypeSelectConfirmCommand`](crate::commit_commands::NodeTypeSelectConfirmCommand).
//!
//! ## Why `Select` is not in the plugin
//!
//! `Select` needs `gpui::Context` and component entities; Ferrum `Plugin::render` only gets [`ferrum_flow::RenderContext`],
//! so the dropdown lives in the window shell (`MeiliShell` in this repo).
//!
//! ## Interaction
//!
//! - Bottom **dropdown** (gpui-component Select) to choose the type; supports built-in search / keyboard navigation.
//! - **Esc** still cancels (handled on canvas focus by this plugin).

use crate::pick_state;
use crate::plugins::pick_link_event::PickNodeTypeForPendingLink;
use ferrum_flow::{
    EventResult, FlowEvent, InputEvent, Plugin, PluginContext, PortPosition, RenderContext,
    RenderLayer, edge_bezier, filled_disc_path,
};
use gpui::{Element as _, ParentElement as _, Styled, canvas, div, px, rgb};

pub struct NodeTypePickerPlugin;

impl NodeTypePickerPlugin {
    pub fn new() -> Self {
        Self
    }

    fn facing_position(p: PortPosition) -> PortPosition {
        match p {
            PortPosition::Left => PortPosition::Right,
            PortPosition::Right => PortPosition::Left,
            PortPosition::Top => PortPosition::Bottom,
            PortPosition::Bottom => PortPosition::Top,
        }
    }
}

impl Plugin for NodeTypePickerPlugin {
    fn name(&self) -> &'static str {
        "meili_node_type_picker"
    }

    fn setup(&mut self, _ctx: &mut ferrum_flow::InitPluginContext) {}

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if let Some(p) = event.as_custom::<PickNodeTypeForPendingLink>() {
            pick_state::pending_set(Some(*p));
            ctx.notify();
            return EventResult::Stop;
        }

        if pick_state::pending_peek().is_none() {
            return EventResult::Continue;
        }

        if let FlowEvent::Input(InputEvent::KeyDown(ev)) = event {
            if ev.keystroke.key == "escape" {
                pick_state::pending_set(None);
                ctx.notify();
                return EventResult::Stop;
            }
        }

        EventResult::Continue
    }

    fn priority(&self) -> i32 {
        130
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    /// Renders only the dangling wire and blue dot; type UI is the gpui-component `Select` in [`crate::shell::MeiliShell`].
    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let p = pick_state::pending_peek()?;
        let port = ctx.graph.get_port(&p.source_port)?;
        let node = ctx.nodes().get(&port.node_id())?;
        let start = ctx.port_screen_center(node, p.source_port)?;
        let end = ctx.world_to_screen(p.end_world);
        let start_position = port.position();
        let target_position = Self::facing_position(start_position);
        let viewport = ctx.viewport().clone();
        let line_rgb = ctx.theme.port_preview_line;
        let dot_rgb = ctx.theme.port_preview_dot;

        let wire = canvas(
            move |_, _, _| (),
            move |_, _, win, _| {
                if let Ok(line) =
                    edge_bezier(start, start_position, target_position, end, &viewport)
                {
                    win.paint_path(line, rgb(line_rgb));
                }
                if let Ok(dot) = filled_disc_path(end, px(6.0)) {
                    win.paint_path(dot, rgb(dot_rgb));
                }
            },
        );

        Some(
            div()
                .absolute()
                .top(px(0.0))
                .left(px(0.0))
                .size_full()
                .child(wire)
                .into_any(),
        )
    }
}
