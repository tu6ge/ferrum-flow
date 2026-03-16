use gpui::{
    AnyElement, KeyDownEvent, KeyUpEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels,
    Point, ScrollWheelEvent, Window,
};

use crate::{
    EdgeId, Graph, NodeId, PortId, Viewport,
    canvas::{CanvasState, Command, History, InteractionHandler, InteractionState},
};

pub trait Plugin {
    fn name(&self) -> &'static str;

    fn setup(&mut self, ctx: &mut InitPluginContext);

    fn on_event(&mut self, _event: &FlowEvent, _context: &mut PluginContext) -> EventResult {
        EventResult::Continue
    }

    fn render(&mut self, _context: &mut RenderContext) -> Option<AnyElement> {
        None
    }

    fn priority(&self) -> i32 {
        0
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
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

    pub history: &'a mut History,
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
        history: &'a mut History,
        emit: &'a mut dyn FnMut(FlowEvent),
        notify: &'a mut dyn FnMut(),
    ) -> Self {
        Self {
            graph,
            viewport,
            interaction,
            history,
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

    pub fn execute_command(&mut self, command: impl Command + 'static) {
        let mut canvas = CanvasState {
            graph: self.graph,
            viewport: self.viewport,
        };

        self.history.execute(Box::new(command), &mut canvas);

        self.notify();
    }

    pub fn undo(&mut self) {
        let mut canvas = CanvasState {
            graph: self.graph,
            viewport: self.viewport,
        };

        self.history.undo(&mut canvas);

        self.notify();
    }

    pub fn redo(&mut self) {
        let mut canvas = CanvasState {
            graph: self.graph,
            viewport: self.viewport,
        };

        self.history.redo(&mut canvas);

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

// TODO
// pub trait Command {
//     fn execute(&mut self, graph: &mut Graph);
//     fn undo(&mut self, graph: &mut Graph);
// }

// pub struct CommandQueue {
//     undo_stack: Vec<Box<dyn Command>>,
//     redo_stack: Vec<Box<dyn Command>>,
// }

pub struct RenderContext<'a> {
    pub graph: &'a Graph,
    pub viewport: &'a Viewport,

    pub window: &'a Window,

    pub layer: RenderLayer,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        graph: &'a Graph,
        viewport: &'a Viewport,
        window: &'a Window,
        layer: RenderLayer,
    ) -> Self {
        Self {
            graph,
            viewport,
            window,
            layer,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RenderLayer {
    Background,
    Edges,
    Nodes,
    Ports,
    Selection,
    Interaction,
    Overlay,
}

impl RenderLayer {
    pub const ALL: [RenderLayer; 7] = [
        RenderLayer::Background,
        RenderLayer::Edges,
        RenderLayer::Nodes,
        RenderLayer::Ports,
        RenderLayer::Selection,
        RenderLayer::Interaction,
        RenderLayer::Overlay,
    ];
    pub fn index(self) -> usize {
        match self {
            RenderLayer::Background => 0,
            RenderLayer::Edges => 1,
            RenderLayer::Nodes => 2,
            RenderLayer::Ports => 3,
            RenderLayer::Selection => 4,
            RenderLayer::Interaction => 5,
            RenderLayer::Overlay => 6,
        }
    }
}

pub struct PluginRegistry {
    pub plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn add(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }
}
