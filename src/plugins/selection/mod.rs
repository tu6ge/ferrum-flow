use std::collections::HashMap;

use gpui::{
    AnyElement, Bounds, Element, MouseMoveEvent, MouseUpEvent, Pixels, Point, Size, Styled, div,
    px, rgb, rgba,
};

use crate::{
    Node, NodeId,
    canvas::{DEFAULT_NODE_HEIGHT, DEFAULT_NODE_WIDTH, InteractionHandler, InteractionResult},
    plugin::{EventResult, FlowEvent, InputEvent, Plugin, PluginContext, RenderContext},
};

const DRAG_THRESHOLD: Pixels = px(2.0);

pub struct SelectionPlugin {}

impl SelectionPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for SelectionPlugin {
    fn name(&self) -> &'static str {
        "selection"
    }
    fn setup(&mut self, ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> EventResult {
        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event {
            if !ev.modifiers.shift {
                ctx.start_interaction(SelectionInteraction::new(ev.position));
                return EventResult::Stop;
            }
        }

        EventResult::Continue
    }

    fn render(
        &mut self,
        _render_ctx: &mut crate::plugin::RenderContext,
        _ctx: &mut gpui::Context<crate::FlowCanvas>,
    ) -> Option<gpui::AnyElement> {
        None
    }
}

pub struct SelectionInteraction {
    state: SelectionState,
}

enum SelectionState {
    Pending {
        start: Point<Pixels>,
    },
    Selecting {
        start: Point<Pixels>,
        end: Point<Pixels>,
    },
    Selected {
        bounds: Bounds<Pixels>,
        nodes: HashMap<NodeId, Point<Pixels>>,
    },
}

impl SelectionInteraction {
    pub fn new(start: Point<Pixels>) -> Self {
        Self {
            state: SelectionState::Pending { start },
        }
    }
}

impl InteractionHandler for SelectionInteraction {
    fn on_mouse_move(&mut self, ev: &MouseMoveEvent, ctx: &mut PluginContext) -> InteractionResult {
        match &mut self.state {
            SelectionState::Pending { start } => {
                let delta = ev.position - *start;

                if delta.x.abs() > DRAG_THRESHOLD && delta.y.abs() > DRAG_THRESHOLD {
                    self.state = SelectionState::Selecting {
                        start: *start,
                        end: ev.position,
                    };

                    ctx.notify();
                }
            }

            SelectionState::Selecting { end, .. } => {
                *end = ev.position;
                ctx.notify();
            }

            SelectionState::Selected { .. } => {}
        }

        InteractionResult::Continue
    }
    fn on_mouse_up(&mut self, _ev: &MouseUpEvent, ctx: &mut PluginContext) -> InteractionResult {
        match &mut self.state {
            SelectionState::Pending { .. } => {
                return InteractionResult::End;
            }

            SelectionState::Selecting { start, end } => {
                let rect = normalize_rect(*start, *end);

                let mut selected = HashMap::new();

                ctx.graph.clear_selected_node();

                for node in ctx.graph.nodes().values() {
                    let bounds = node_screen_bounds(node, ctx);

                    if rect.intersects(&bounds) {
                        selected.insert(node.id, node.point());
                    }
                }

                for (id, _) in selected.iter() {
                    ctx.graph.add_selected_node(*id, true);
                }

                self.state = SelectionState::Selected {
                    bounds: rect,
                    nodes: selected,
                };

                ctx.notify();

                return InteractionResult::Continue;
            }

            SelectionState::Selected { .. } => {}
        }

        InteractionResult::Continue
    }
    fn render(&self, _ctx: &mut RenderContext) -> Option<AnyElement> {
        match &self.state {
            SelectionState::Selecting { start, end } => {
                let rect = normalize_rect(*start, *end);

                Some(render_rect(rect))
            }

            SelectionState::Selected { bounds, .. } => Some(render_rect(*bounds)),

            _ => None,
        }
    }
}

fn normalize_rect(start: Point<Pixels>, end: Point<Pixels>) -> Bounds<Pixels> {
    let x = start.x.min(end.x);
    let y = start.y.min(end.y);

    let w = (end.x - start.x).abs();
    let h = (end.y - start.y).abs();

    Bounds::new(Point::new(x, y), Size::new(w, h))
}

fn render_rect(bounds: Bounds<Pixels>) -> AnyElement {
    div()
        .absolute()
        .left(bounds.origin.x)
        .top(bounds.origin.y)
        .w(bounds.size.width)
        .h(bounds.size.height)
        .border(px(1.0))
        .border_color(rgb(0x78A0FF))
        .bg(rgba(0x78A0FF4c))
        .into_any()
}

fn node_screen_bounds(node: &Node, ctx: &PluginContext) -> Bounds<Pixels> {
    let pos = ctx.viewport.screen_to_world(Point::new(node.x, node.y));

    let w = DEFAULT_NODE_WIDTH * ctx.viewport.zoom;
    let h = DEFAULT_NODE_HEIGHT * ctx.viewport.zoom;

    Bounds::new(pos, Size::new(w, h))
}
