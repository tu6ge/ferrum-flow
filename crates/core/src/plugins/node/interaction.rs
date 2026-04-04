use gpui::{Pixels, Point, px};

use crate::{
    NodeId,
    canvas::{Interaction, InteractionResult},
    plugin::{EventResult, FlowEvent, InitPluginContext, InputEvent, Plugin, PluginContext},
    plugins::node::command::{DragNodesCommand, SelecteNodeCommand},
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
            let mouse_world = ctx.screen_to_world(ev.position);

            if let Some(node_id) = ctx.hit_node(mouse_world) {
                ctx.start_interaction(NodeDragInteraction::start(
                    node_id,
                    mouse_world,
                    ev.modifiers.shift,
                ));

                return EventResult::Stop;
            } else {
                ctx.clear_selected_node();
            }
        }

        EventResult::Continue
    }

    fn priority(&self) -> i32 {
        120
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

impl Interaction for NodeDragInteraction {
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
                let delta = ctx.screen_to_world(ev.position) - *start_mouse;
                if delta.x.abs() > DRAG_THRESHOLD || delta.y.abs() > DRAG_THRESHOLD {
                    let mut nodes = vec![];

                    if ctx.graph.selected_node.contains(&node_id) {
                        for id in &ctx.graph.selected_node {
                            if let Some(node) = ctx.nodes().get(&id) {
                                nodes.push((id.clone(), node.point()));
                            }
                        }
                    } else {
                        if let Some(node) = ctx.nodes().get(&node_id) {
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
                    if let Some(node) = ctx.get_node_mut(id) {
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
                ctx.execute_command(SelecteNodeCommand::new(*node_id, *shift, ctx));
                InteractionResult::End
            }
            NodeDragState::Draging {
                start_positions, ..
            } => {
                ctx.execute_command(DragNodesCommand::new(start_positions, &ctx));
                InteractionResult::End
            }
        }
    }
    fn render(&self, _ctx: &mut crate::plugin::RenderContext) -> Option<gpui::AnyElement> {
        None
    }
}
