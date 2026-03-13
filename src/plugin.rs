use gpui::{
    AnyElement, Context, KeyDownEvent, KeyUpEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, ScrollWheelEvent,
};

use crate::{
    EdgeId, FlowCanvas, Graph, Node, NodeId, PortId, Viewport,
    canvas::{InteractionHandler, InteractionState},
};

pub trait Plugin {
    fn name(&self) -> &'static str;

    fn setup(&mut self, ctx: &mut InitPluginContext);

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        EventResult::Continue
    }

    fn render(
        &mut self,
        render_ctx: &mut RenderContext,
        ctx: &mut Context<FlowCanvas>,
    ) -> Option<AnyElement> {
        None
    }

    fn priority(&self) -> i32 {
        0
    }
}

pub struct InitPluginContext<'a> {
    pub graph: &'a mut Graph,
    pub viewport: &'a mut Viewport,
}

pub struct PluginContext<'a> {
    pub graph: &'a mut Graph,
    pub viewport: &'a mut Viewport,
    pub(crate) interaction: &'a mut InteractionState,

    //pub commands: &'a mut CommandQueue,
    emit: &'a mut dyn FnMut(FlowEvent),
    notify: &'a mut dyn FnMut(),
}

pub enum EventResult {
    Continue,
    Stop,
}

impl<'a> PluginContext<'a> {
    pub fn new(
        graph: &'a mut Graph,
        viewport: &'a mut Viewport,
        interaction: &'a mut InteractionState,
        emit: &'a mut dyn FnMut(FlowEvent),
        notify: &'a mut dyn FnMut(),
    ) -> Self {
        Self {
            graph,
            viewport,
            interaction,
            //commands: ,
            emit,
            notify,
        }
    }

    pub fn start_interaction(&mut self, handler: impl InteractionHandler + 'static) {
        self.interaction.handler = Some(Box::new(handler));
    }

    pub fn cancel_interaction(&mut self) {
        self.interaction.handler = None;
    }

    pub fn has_interaction(&self) -> bool {
        self.interaction.handler.is_some()
    }

    // pub fn take_interaction(&mut self) -> Option<Box<dyn InteractionHandler + 'static>> {
    //     self.interaction.handler.take()
    // }

    pub fn notify(&mut self) {
        (self.notify)();
    }

    pub fn emit(&mut self, event: FlowEvent) {
        (self.emit)(event);
        self.notify();
    }
}

pub enum FlowEvent {
    Input(InputEvent),
    Graph(GraphEvent),
    Ui(UiEvent),
    Custom(Box<dyn std::any::Any + Send>),
}

impl FlowEvent {
    pub fn custom<T: 'static + Send>(event: T) -> Self {
        FlowEvent::Custom(Box::new(event))
    }
    pub fn as_custom<T: 'static>(&self) -> Option<&T> {
        match self {
            FlowEvent::Custom(e) => e.downcast_ref::<T>(),
            _ => None,
        }
    }
}

pub enum InputEvent {
    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),

    MouseDown(MouseDownEvent),
    MouseMove(MouseMoveEvent),
    MouseUp(MouseUpEvent),

    Wheel(ScrollWheelEvent),
}

pub enum GraphEvent {
    NodeClicked(NodeId),

    NodeDragStart(NodeId),
    NodeDragging(NodeId, Point<Pixels>),
    NodeDragEnd(NodeId),

    EdgeClicked(EdgeId),

    EdgeCreated { from: PortId, to: PortId },

    EdgeRemoved(EdgeId),
}

pub enum UiEvent {
    SelectionChanged(Vec<NodeId>),

    ConnectStart(PortId),

    ConnectPreview(Point<Pixels>),

    ConnectEnd(PortId),

    ConnectCancel,

    ViewportChanged { zoom: f32, pan: Point<Pixels> },
}

pub trait Command {
    fn execute(&mut self, graph: &mut Graph);
    fn undo(&mut self, graph: &mut Graph);
}

pub struct CommandQueue {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

pub struct RenderContext<'a> {
    pub graph: &'a Graph,
    pub viewport: &'a Viewport,

    pub layer: RenderLayer,
}

impl<'a> RenderContext<'a> {
    pub fn new(graph: &'a Graph, viewport: &'a Viewport, layer: RenderLayer) -> Self {
        Self {
            graph,
            viewport,
            layer,
        }
    }
}

pub enum RenderLayer {
    Background,
    Edges,
    Nodes,
    Overlay,
}

pub struct NodeRenderContext<'a> {
    pub node: &'a Node,
    pub selected: bool,
    pub hovered: bool,
    pub viewport: &'a Viewport,
    pub cx: &'a mut Context<'a, FlowCanvas>,
}
