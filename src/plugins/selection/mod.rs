use std::collections::HashMap;

use gpui::{
    Bounds, Element, MouseMoveEvent, MouseUpEvent, Pixels, Point, Size, Styled, div, px, rgb, rgba,
};

use crate::{
    Node, NodeId,
    canvas::{DEFAULT_NODE_HEIGHT, DEFAULT_NODE_WIDTH, InteractionHandler, InteractionResult},
    plugin::{EventResult, FlowEvent, InputEvent, Plugin, PluginContext},
};

const DRAG_THRESHOLD: Pixels = px(2.0);

pub struct SelectionPlugin {
    box_selection: Option<BoxSelection>,
}

impl SelectionPlugin {
    pub fn new() -> Self {
        Self {
            box_selection: None,
        }
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
        match &event {
            FlowEvent::Input(InputEvent::MouseDown(ev)) => {
                if !ev.modifiers.shift {
                    ctx.start_interaction(PendingBoxSelect { start: ev.position });
                    ctx.notify();
                    return EventResult::Stop;
                }
            }
            FlowEvent::Custom(cus) => {
                if let Some(_) = cus.downcast_ref::<BoxSelectClear>() {
                    self.box_selection = None;
                    return EventResult::Stop;
                }
                if let Some(data) = cus.downcast_ref::<BoxSelection>() {
                    self.box_selection = Some(data.clone());
                };
            }
            FlowEvent::Input(InputEvent::MouseUp(_)) => {
                if self.box_selection.is_some() {
                    self.box_selection = None;
                    ctx.notify();
                }
            }
            _ => {}
        }

        EventResult::Continue
    }

    fn render(
        &mut self,
        _render_ctx: &mut crate::plugin::RenderContext,
        _ctx: &mut gpui::Context<crate::FlowCanvas>,
    ) -> Option<gpui::AnyElement> {
        self.box_selection
            .as_ref()
            .map(|BoxSelection { bounds, .. }| {
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
            })
    }
}

#[derive(Debug, Clone)]
pub struct BoxSelection {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) bounds: Bounds<Pixels>,
    pub(super) nodes: HashMap<NodeId, Point<Pixels>>,
}

#[derive(Debug, Clone)]
pub struct PendingBoxSelect {
    pub(super) start: Point<Pixels>,
}

#[derive(Debug, Clone)]
pub struct BoxSelectDrag {
    pub(super) start: Point<Pixels>,
    pub(super) end: Point<Pixels>,
}

pub struct BoxMoveDrag {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) start_bounds: Bounds<Pixels>,
    pub(super) nodes: Vec<(NodeId, Point<Pixels>)>,
}

impl InteractionHandler for PendingBoxSelect {
    fn on_mouse_move(&mut self, ev: &MouseMoveEvent, ctx: &mut PluginContext) -> InteractionResult {
        let delta = ev.position - self.start;
        if delta.x > DRAG_THRESHOLD && delta.y > DRAG_THRESHOLD {
            ctx.emit(FlowEvent::Custom(Box::new(BoxSelectClear {})));
            InteractionResult::Replace(Box::new(BoxSelectDrag {
                start: self.start,
                end: ev.position,
            }))
        } else {
            InteractionResult::Continue
        }
    }
    fn on_mouse_up(&mut self, _: &MouseUpEvent, ctx: &mut PluginContext) -> InteractionResult {
        ctx.emit(FlowEvent::Custom(Box::new(BoxSelectClear {})));
        InteractionResult::End
    }
}

impl InteractionHandler for BoxSelectDrag {
    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        ctx: &mut PluginContext,
    ) -> InteractionResult {
        self.end = event.position;
        ctx.notify();
        InteractionResult::Continue
    }
    fn on_mouse_up(&mut self, _event: &MouseUpEvent, ctx: &mut PluginContext) -> InteractionResult {
        self.finalize_selection(ctx);
        ctx.notify();
        InteractionResult::End
    }
    fn render(&self, _ctx: &mut crate::plugin::RenderContext) -> Option<gpui::AnyElement> {
        Some(
            div()
                .absolute()
                .left(self.start.x)
                .top(self.start.y)
                .w(self.end.x - self.start.x)
                .h(self.end.y - self.start.y)
                .border(px(1.0))
                .border_color(rgb(0x78A0FF))
                .bg(rgba(0x78A0FF4c))
                .into_any(),
        )
    }
}

impl BoxSelectDrag {
    fn selection_bounds_mouse(&self) -> Bounds<Pixels> {
        let size = Size::new(self.end.x - self.start.x, self.end.y - self.start.y);
        Bounds::new(self.start, size)
    }
    fn node_screen_bounds(&self, node: &Node, ctx: &PluginContext) -> Bounds<Pixels> {
        let pos = ctx.viewport.screen_to_world(Point::new(node.x, node.y));

        let w = DEFAULT_NODE_WIDTH * ctx.viewport.zoom;
        let h = DEFAULT_NODE_HEIGHT * ctx.viewport.zoom;

        Bounds::new(pos, Size::new(w, h))
    }
    fn finalize_selection(&mut self, ctx: &mut PluginContext) {
        let rect = self.selection_bounds_mouse();
        ctx.graph.clear_selected_node();
        let mut selected_ids = HashMap::new();

        for node in ctx.graph.nodes().values() {
            let pos = self.node_screen_bounds(node, ctx);

            if rect.intersects(&pos) {
                selected_ids.insert(node.id, node.point());
            }
        }

        for (id, _) in selected_ids.iter() {
            ctx.graph.add_selected_node(*id, true);
        }

        ctx.emit(FlowEvent::custom(BoxSelection {
            start_mouse: self.start,
            bounds: rect,
            nodes: selected_ids,
        }));
    }
}

impl InteractionHandler for BoxMoveDrag {
    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        ctx: &mut PluginContext,
    ) -> InteractionResult {
        todo!()
    }
    fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut PluginContext) -> InteractionResult {
        todo!()
    }
}

struct BoxSelectClear {}
