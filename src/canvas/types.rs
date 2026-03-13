use gpui::{AnyElement, MouseMoveEvent, MouseUpEvent, Pixels, Point};

use crate::{
    NodeId, PortId,
    plugin::{PluginContext, RenderContext},
};

#[derive(Debug, Clone)]
pub enum DragState {
    None,
    NodeDrag(NodeDrag),
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

    pub fn add(&mut self, handler: impl InteractionHandler + 'static) {
        self.handler = Some(Box::new(handler));
    }

    pub fn clear(&mut self) {
        self.handler = None;
    }

    pub fn is_some(&self) -> bool {
        self.handler.is_some()
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
    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        ctx: &mut PluginContext,
    ) -> InteractionResult;

    fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut PluginContext) -> InteractionResult;

    fn render(&self, ctx: &mut RenderContext) -> Option<AnyElement> {
        None
    }
}

pub enum InteractionResult {
    Continue,
    End,
    Replace(Box<dyn InteractionHandler>),
}

#[derive(Debug, Clone)]
pub struct NodeDrag {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) start_positions: Vec<(NodeId, Point<Pixels>)>,
}

// impl InteractionHandler for NodeDrag {
//     fn on_mouse_move(
//         &mut self,
//         event: &MouseMoveEvent,
//         ctx: &mut DragContext,
//     ) -> InteractionResult {
//         todo!()
//     }
//     fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut DragContext) -> InteractionResult {
//         todo!()
//     }
// }

#[derive(Debug, Clone)]
pub struct PendingNode {
    pub(super) node_id: NodeId,
    pub(super) start_mouse: Point<Pixels>,
    pub(super) shift: bool,
}

// impl InteractionHandler for PendingNode {
//     fn on_mouse_move(&mut self, ev: &MouseMoveEvent, ctx: &mut DragContext) -> InteractionResult {
//         let node = &ctx.graph.nodes()[&self.node_id];
//         let delta = ev.position - self.start_mouse;
//         if delta.x > DRAG_THRESHOLD || delta.y > DRAG_THRESHOLD {
//             InteractionResult::Replace(Box::new(NodeDrag {
//                 start_mouse: ev.position,
//                 start_positions: vec![(self.node_id.clone(), node.point())],
//             }))
//         } else {
//             InteractionResult::Continue
//         }
//     }
//     fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut DragContext) -> InteractionResult {
//         todo!()
//     }
// }

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
