use gpui::{
    AnyElement, Context, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point,
};

use crate::{
    EdgeId, FlowCanvas, Graph, Node, NodeId, PortId, Viewport,
    canvas::{InteractionHandler, InteractionState},
};

pub trait Plugin {
    fn name(&self) -> &'static str;

    fn setup(&mut self, ctx: &mut PluginContext);

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext);

    fn render(&mut self, ctx: &mut RenderContext) -> Option<AnyElement> {
        None
    }

    fn priority(&self) -> i32 {
        0
    }
}

pub struct PluginContext<'a> {
    pub graph: &'a mut Graph,
    pub viewport: &'a mut Viewport,
    pub plugins: &'a mut Vec<Box<dyn Plugin>>,
    interaction: &'a mut InteractionState,
    pub commands: &'a mut CommandQueue,
    pub emit: &'a dyn Fn(FlowEvent),
}

impl<'a> PluginContext<'a> {
    pub fn start_interaction(&mut self, handler: impl InteractionHandler + 'static) {
        self.interaction.handler = Some(Box::new(handler));
    }

    pub fn cancel_interaction(&mut self) {
        self.interaction.handler = None;
    }

    pub fn has_interaction(&self) -> bool {
        self.interaction.handler.is_some()
    }
}

pub enum FlowEvent {
    NodeClicked(NodeId),
    NodeDragged(NodeId, Point<Pixels>),

    EdgeClicked(EdgeId),

    SelectionChanged(Vec<NodeId>),

    ConnectStart(PortId),
    ConnectEnd(PortId),

    ViewportChanged,

    KeyDown(KeyDownEvent),
    MouseDown(MouseDownEvent),
    MouseMove(MouseMoveEvent),
    MouseUp(MouseUpEvent),
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

    pub cx: &'a mut Context<'a, FlowCanvas>,

    pub screen_to_world: fn(Point<Pixels>) -> Point<Pixels>,
    pub world_to_screen: fn(Point<Pixels>) -> Point<Pixels>,
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
