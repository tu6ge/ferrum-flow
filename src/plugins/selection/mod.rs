use std::collections::HashMap;

use gpui::{
    AnyElement, Bounds, Element, MouseMoveEvent, MouseUpEvent, Pixels, Point, Size, Styled, div,
    px, rgb, rgba,
};

use crate::{
    Graph, Node, NodeId,
    canvas::{DEFAULT_NODE_HEIGHT, DEFAULT_NODE_WIDTH, InteractionHandler, InteractionResult},
    plugin::{
        EventResult, FlowEvent, InputEvent, Plugin, PluginContext, RenderContext, RenderLayer,
    },
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
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> EventResult {
        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event {
            if !ev.modifiers.shift {
                let start = ctx.viewport.screen_to_world(ev.position);
                if let Some(selection) = ctx.graph.selection_bounds() {
                    if selection.contains(&start) {
                        let nodes = ctx.graph.selected_nodes_with_positions();

                        ctx.start_interaction(SelectionInteraction::start_move(
                            start, selection, nodes,
                        ));

                        return EventResult::Stop;
                    }
                }

                ctx.start_interaction(SelectionInteraction::new(start));
                return EventResult::Stop;
            }
        }

        EventResult::Continue
    }
    fn priority(&self) -> i32 {
        50
    }
    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Selection
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
    },
    Moving {
        start_mouse: Point<Pixels>,
        start_bounds: Bounds<Pixels>,
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
    pub fn start_move(
        mouse: Point<Pixels>,
        bounds: Bounds<Pixels>,
        nodes: HashMap<NodeId, Point<Pixels>>,
    ) -> Self {
        Self {
            state: SelectionState::Moving {
                start_mouse: mouse,
                start_bounds: bounds.clone(),
                bounds: bounds,
                nodes,
            },
        }
    }
}

impl InteractionHandler for SelectionInteraction {
    fn on_mouse_move(&mut self, ev: &MouseMoveEvent, ctx: &mut PluginContext) -> InteractionResult {
        let mouse_world = ctx.viewport.screen_to_world(ev.position);
        match &mut self.state {
            SelectionState::Pending { start } => {
                let delta = mouse_world - *start;

                if delta.x.abs() > DRAG_THRESHOLD && delta.y.abs() > DRAG_THRESHOLD {
                    self.state = SelectionState::Selecting {
                        start: *start,
                        end: mouse_world,
                    };

                    ctx.notify();
                }
            }

            SelectionState::Selecting { end, .. } => {
                *end = mouse_world;
                ctx.notify();
            }

            SelectionState::Selected { .. } => {}

            SelectionState::Moving {
                start_mouse,
                start_bounds,
                bounds,
                nodes,
            } => {
                let delta = mouse_world - *start_mouse;

                for (id, start_pos) in nodes.iter() {
                    if let Some(node) = ctx.graph.get_node_mut(*id) {
                        node.x = start_pos.x + delta.x;
                        node.y = start_pos.y + delta.y;
                    }
                }
                *bounds = Bounds::new(start_bounds.origin + delta, start_bounds.size);

                ctx.notify();
            }
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
                    let bounds = node_world_bounds(node);

                    if rect.intersects(&bounds) {
                        selected.insert(node.id, node.point());
                    }
                }

                for (id, _) in selected.iter() {
                    ctx.graph.add_selected_node(*id, true);
                }

                let bounds = compute_nodes_bounds(&selected, ctx.graph);

                self.state = SelectionState::Selected { bounds };

                ctx.notify();

                return InteractionResult::Continue;
            }

            SelectionState::Selected { .. } => {}
            SelectionState::Moving { bounds, nodes, .. } => {
                let bounds = *bounds;

                for (id, _) in nodes.iter() {
                    ctx.graph.add_selected_node(*id, true);
                }

                self.state = SelectionState::Selected { bounds };

                ctx.notify();
            }
        }

        InteractionResult::Continue
    }
    fn render(&self, ctx: &mut RenderContext) -> Option<AnyElement> {
        match &self.state {
            SelectionState::Selecting { start, end } => {
                let rect = normalize_rect(*start, *end);

                let top_left = ctx.viewport.world_to_screen(rect.origin);

                let size = Size::new(
                    rect.size.width * ctx.viewport.zoom,
                    rect.size.height * ctx.viewport.zoom,
                );

                Some(render_rect(Bounds::new(top_left, size)))
            }

            SelectionState::Selected { bounds, .. } => {
                let top_left = ctx.viewport.world_to_screen(bounds.origin);

                let size = Size::new(
                    bounds.size.width * ctx.viewport.zoom,
                    bounds.size.height * ctx.viewport.zoom,
                );
                Some(render_rect(Bounds::new(top_left, size)))
            }
            SelectionState::Moving { bounds, .. } => {
                let top_left = ctx.viewport.world_to_screen(bounds.origin);

                let size = Size::new(
                    bounds.size.width * ctx.viewport.zoom,
                    bounds.size.height * ctx.viewport.zoom,
                );
                Some(render_rect(Bounds::new(top_left, size)))
            }

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

fn node_world_bounds(node: &Node) -> Bounds<Pixels> {
    Bounds::new(
        Point::new(node.x, node.y),
        Size::new(DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT),
    )
}

fn compute_nodes_bounds(nodes: &HashMap<NodeId, Point<Pixels>>, graph: &Graph) -> Bounds<Pixels> {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for id in nodes.keys() {
        let node = &graph.nodes()[id];

        min_x = min_x.min(node.x.into());
        min_y = min_y.min(node.y.into());

        max_x = max_x.max((node.x + DEFAULT_NODE_WIDTH).into());
        max_y = max_y.max((node.y + DEFAULT_NODE_HEIGHT).into());
    }

    Bounds::new(
        Point::new(px(min_x), px(min_y)),
        Size::new(px(max_x - min_x), px(max_y - min_y)),
    )
}
