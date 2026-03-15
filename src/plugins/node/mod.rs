use gpui::{Pixels, Point, px};

use crate::{
    NodeId,
    canvas::{InteractionHandler, InteractionResult},
    plugin::{EventResult, FlowEvent, InitPluginContext, InputEvent, Plugin, PluginContext},
};

const DRAG_THRESHOLD: Pixels = px(2.0);

pub struct NodeInteractionPlugin;

impl NodeInteractionPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for NodeInteractionPlugin {
    fn name(&self) -> &'static str {
        "node_interaction"
    }

    fn setup(&mut self, _ctx: &mut InitPluginContext) {}

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event {
            let mouse_world = ctx.viewport.screen_to_world(ev.position);

            if let Some(node_id) = ctx.graph.hit_node(mouse_world) {
                ctx.start_interaction(NodeDragInteraction::start(
                    node_id,
                    mouse_world,
                    ev.modifiers.shift,
                ));

                return EventResult::Stop;
            } else {
                ctx.graph.clear_selected_node();
            }
        }

        EventResult::Continue
    }

    fn priority(&self) -> i32 {
        100
    }
}

pub struct NodeDragInteraction {
    state: NodeDragState,
}

enum NodeDragState {
    Pending {
        node_id: NodeId,
        start_mouse: Point<Pixels>,
        shift: bool,
    },
    Draging {
        start_mouse: Point<Pixels>,
        start_positions: Vec<(NodeId, Point<Pixels>)>,
    },
}

impl NodeDragInteraction {
    fn start(node_id: NodeId, start_mouse: Point<Pixels>, shift: bool) -> Self {
        Self {
            state: NodeDragState::Pending {
                node_id,
                start_mouse,
                shift,
            },
        }
    }
}

impl InteractionHandler for NodeDragInteraction {
    fn on_mouse_move(
        &mut self,
        ev: &gpui::MouseMoveEvent,
        ctx: &mut PluginContext,
    ) -> crate::canvas::InteractionResult {
        match &self.state {
            NodeDragState::Pending {
                node_id,
                start_mouse,
                ..
            } => {
                let delta = ctx.viewport.screen_to_world(ev.position) - *start_mouse;
                if delta.x.abs() > DRAG_THRESHOLD || delta.y.abs() > DRAG_THRESHOLD {
                    let mut nodes = vec![];

                    if ctx.graph.selected_node.contains(&node_id) {
                        for id in &ctx.graph.selected_node {
                            if let Some(node) = ctx.graph.nodes().get(&id) {
                                nodes.push((id.clone(), node.point()));
                            }
                        }
                    } else {
                        if let Some(node) = ctx.graph.nodes().get(&node_id) {
                            nodes.push((node_id.clone(), node.point()));
                        }
                    }
                    self.state = NodeDragState::Draging {
                        start_mouse: ev.position,
                        start_positions: nodes,
                    };

                    ctx.notify();
                }
            }
            NodeDragState::Draging {
                start_mouse,
                start_positions,
            } => {
                let dx = (ev.position.x - start_mouse.x) / ctx.viewport.zoom;
                let dy = (ev.position.y - start_mouse.y) / ctx.viewport.zoom;
                for (id, point) in start_positions.iter() {
                    if let Some(node) = ctx.graph.get_node_mut(*id) {
                        node.x = point.x + dx;
                        node.y = point.y + dy;
                    }
                }
                ctx.notify();
            }
        }
        InteractionResult::Continue
    }
    fn on_mouse_up(
        &mut self,
        _ev: &gpui::MouseUpEvent,
        ctx: &mut PluginContext,
    ) -> crate::canvas::InteractionResult {
        match &self.state {
            NodeDragState::Pending { node_id, shift, .. } => {
                if !shift {
                    ctx.graph.clear_selected_edge();
                }
                ctx.graph.add_selected_node(*node_id, *shift);
                ctx.graph.bring_node_to_front(*node_id);
                ctx.notify();
                InteractionResult::End
            }
            NodeDragState::Draging { .. } => InteractionResult::End,
        }
    }
    fn render(&self, _ctx: &mut crate::plugin::RenderContext) -> Option<gpui::AnyElement> {
        None
    }
}
