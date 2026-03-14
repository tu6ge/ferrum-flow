use gpui::{AnyElement, MouseMoveEvent, MouseUpEvent, Pixels, Point};

use crate::{
    NodeId, PortId,
    plugin::{PluginContext, RenderContext},
};

#[derive(Debug, Clone)]
pub enum DragState {
    None,
    Pan(Panning),
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

pub trait InteractionHandler {
    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        ctx: &mut PluginContext,
    ) -> InteractionResult;

    fn on_mouse_up(&mut self, event: &MouseUpEvent, ctx: &mut PluginContext) -> InteractionResult;

    fn render(&self, _ctx: &mut RenderContext) -> Option<AnyElement> {
        None
    }
}

pub enum InteractionResult {
    Continue,
    End,
    Replace(Box<dyn InteractionHandler>),
}

impl InteractionResult {
    pub fn replace(new_handler: impl InteractionHandler + 'static) -> Self {
        Self::Replace(Box::new(new_handler))
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
