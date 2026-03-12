use std::collections::HashMap;

use gpui::{AnyElement, Bounds, MouseMoveEvent, MouseUpEvent, Pixels, Point, px};

use crate::{
    NodeId, PortId,
    plugin::{PluginContext, RenderContext},
};

const DRAG_THRESHOLD: Pixels = px(2.0);

#[derive(Debug, Clone)]
pub enum DragState {
    None,
    NodeDrag(NodeDrag),
    PendingBoxSelect(PendingBoxSelect),
    BoxSelect(BoxSelectDrag),
    BoxMove(BoxMoveDrag),
    Pan(Panning),
    PendingNode(PendingNode),
    EdgeDrag(Connecting),
}

pub struct InteractionState {
    pub handler: Option<Box<dyn InteractionHandler>>,
}

impl InteractionState {
    pub fn new() -> Self {
        Self { handler: None }
    }
}

// if let Some(handler) = &mut self.interaction.handler {

//     handler.on_mouse_move(...)

// } else {

//     for plugin in plugins {
//         plugin.on_event(...)
//     }

// }
pub trait InteractionHandler {
    fn on_mouse_move(&mut self, event: &MouseMoveEvent, ctx: &mut PluginContext);

    fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut PluginContext);

    fn render(&self, ctx: &mut RenderContext) -> Option<AnyElement> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct NodeDrag {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) start_positions: Vec<(NodeId, Point<Pixels>)>,
}

impl InteractionHandler for NodeDrag {
    fn on_mouse_move(&mut self, event: &MouseMoveEvent, ctx: &mut PluginContext) {
        todo!()
    }
    fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut PluginContext) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct BoxMoveDrag {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) start_bounds: Bounds<Pixels>,
    pub(super) nodes: Vec<(NodeId, Point<Pixels>)>,
}

#[derive(Debug, Clone)]
pub struct BoxSelection {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) bounds: Bounds<Pixels>,
    pub(super) nodes: HashMap<NodeId, Point<Pixels>>,
}

#[derive(Debug, Clone)]
pub struct PendingNode {
    pub(super) node_id: NodeId,
    pub(super) start_mouse: Point<Pixels>,
    pub(super) shift: bool,
}

impl InteractionHandler for PendingNode {
    fn on_mouse_move(&mut self, ev: &MouseMoveEvent, ctx: &mut PluginContext) {
        let node = &ctx.graph.nodes()[&self.node_id];
        let delta = ev.position - self.start_mouse;
        if delta.x > DRAG_THRESHOLD || delta.y > DRAG_THRESHOLD {
            ctx.start_interaction(NodeDrag {
                start_mouse: ev.position,
                start_positions: vec![(self.node_id.clone(), node.point())],
            });
        }
    }
    fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut PluginContext) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Connecting {
    pub(super) node_id: NodeId,
    pub(super) port_id: PortId,
    pub(super) mouse: Point<Pixels>,
}

#[derive(Debug, Clone)]
pub struct Panning {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) start_offset: Point<Pixels>,
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
