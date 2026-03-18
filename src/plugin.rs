use std::collections::HashMap;

use gpui::{
    AnyElement, Bounds, KeyDownEvent, KeyUpEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, ScrollWheelEvent, Window,
};

use crate::{
    Edge, EdgeId, Graph, Node, NodeBuilder, NodeId, Port, PortId, Viewport,
    canvas::{CanvasState, Command, History, Interaction, InteractionState},
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

impl<'a> InitPluginContext<'a> {
    pub fn create_node(&self, node_type: &str) -> NodeBuilder {
        self.graph.create_node(node_type)
    }

    pub fn next_node_id(&self) -> NodeId {
        self.graph.next_node_id()
    }

    pub fn next_port_id(&self) -> PortId {
        self.graph.next_port_id()
    }

    pub fn next_edge_id(&self) -> EdgeId {
        self.graph.next_edge_id()
    }

    pub fn add_node(&mut self, node: Node) {
        self.graph.add_node(node);
    }

    pub fn add_point(&mut self, port: Port) {
        self.graph.add_point(port);
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.graph.get_node(id)
    }

    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.graph.get_node_mut(id)
    }
    pub fn remove_node(&mut self, id: &NodeId) {
        self.graph.remove_node(id);
    }
    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        self.graph.nodes()
    }
    pub fn node_order(&self) -> &Vec<NodeId> {
        self.graph.node_order()
    }

    pub fn new_edge(&self) -> Edge {
        self.graph.new_edge()
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.graph.add_edge(edge);
    }

    pub fn remove_edge(&mut self, edge_id: EdgeId) {
        self.graph.remove_edge(edge_id);
    }

    pub fn add_selected_node(&mut self, id: NodeId, shift: bool) {
        self.graph.add_selected_node(id, shift);
    }
    pub fn clear_selected_node(&mut self) {
        self.graph.clear_selected_node();
    }
    pub fn remove_selected_node(&mut self) -> bool {
        self.graph.remove_selected_node()
    }

    pub fn add_selected_edge(&mut self, id: EdgeId, shift: bool) {
        self.graph.add_selected_edge(id, shift);
    }
    pub fn clear_selected_edge(&mut self) {
        self.graph.clear_selected_edge();
    }
    pub fn remove_selected_edge(&mut self) -> bool {
        self.graph.remove_selected_edge()
    }

    pub fn selection_bounds(&self) -> Option<Bounds<Pixels>> {
        self.graph.selection_bounds()
    }

    pub fn selected_nodes_with_positions(&self) -> HashMap<NodeId, Point<Pixels>> {
        self.graph.selected_nodes_with_positions()
    }

    pub fn hit_node(&self, mouse: Point<Pixels>) -> Option<NodeId> {
        self.graph.hit_node(mouse)
    }

    pub fn bring_node_to_front(&mut self, node_id: NodeId) {
        self.graph.bring_node_to_front(node_id);
    }

    pub fn world_to_screen(&self, p: Point<Pixels>) -> Point<Pixels> {
        self.viewport.world_to_screen(p)
    }

    pub fn screen_to_world(&self, p: Point<Pixels>) -> Point<Pixels> {
        self.viewport.screen_to_world(p)
    }
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

    pub fn start_interaction(&mut self, handler: impl Interaction + 'static) {
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

    pub fn create_node(&self, node_type: &str) -> NodeBuilder {
        self.graph.create_node(node_type)
    }

    pub fn next_node_id(&self) -> NodeId {
        self.graph.next_node_id()
    }

    pub fn next_port_id(&self) -> PortId {
        self.graph.next_port_id()
    }

    pub fn next_edge_id(&self) -> EdgeId {
        self.graph.next_edge_id()
    }

    pub fn add_node(&mut self, node: Node) {
        self.graph.add_node(node);
    }

    pub fn add_point(&mut self, port: Port) {
        self.graph.add_point(port);
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.graph.get_node(id)
    }

    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.graph.get_node_mut(id)
    }
    pub fn remove_node(&mut self, id: &NodeId) {
        self.graph.remove_node(id);
    }
    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        self.graph.nodes()
    }
    pub fn node_order(&self) -> &Vec<NodeId> {
        self.graph.node_order()
    }

    pub fn new_edge(&self) -> Edge {
        self.graph.new_edge()
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.graph.add_edge(edge);
    }

    pub fn remove_edge(&mut self, edge_id: EdgeId) {
        self.graph.remove_edge(edge_id);
    }

    pub fn add_selected_node(&mut self, id: NodeId, shift: bool) {
        self.graph.add_selected_node(id, shift);
    }
    pub fn clear_selected_node(&mut self) {
        self.graph.clear_selected_node();
    }
    pub fn remove_selected_node(&mut self) -> bool {
        self.graph.remove_selected_node()
    }

    pub fn add_selected_edge(&mut self, id: EdgeId, shift: bool) {
        self.graph.add_selected_edge(id, shift);
    }
    pub fn clear_selected_edge(&mut self) {
        self.graph.clear_selected_edge();
    }
    pub fn remove_selected_edge(&mut self) -> bool {
        self.graph.remove_selected_edge()
    }

    pub fn selection_bounds(&self) -> Option<Bounds<Pixels>> {
        self.graph.selection_bounds()
    }

    pub fn selected_nodes_with_positions(&self) -> HashMap<NodeId, Point<Pixels>> {
        self.graph.selected_nodes_with_positions()
    }

    pub fn hit_node(&self, mouse: Point<Pixels>) -> Option<NodeId> {
        self.graph.hit_node(mouse)
    }

    pub fn bring_node_to_front(&mut self, node_id: NodeId) {
        self.graph.bring_node_to_front(node_id);
    }

    pub fn world_to_screen(&self, p: Point<Pixels>) -> Point<Pixels> {
        self.viewport.world_to_screen(p)
    }

    pub fn screen_to_world(&self, p: Point<Pixels>) -> Point<Pixels> {
        self.viewport.screen_to_world(p)
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

    pub fn create_node(&self, node_type: &str) -> NodeBuilder {
        self.graph.create_node(node_type)
    }

    pub fn next_node_id(&self) -> NodeId {
        self.graph.next_node_id()
    }

    pub fn next_port_id(&self) -> PortId {
        self.graph.next_port_id()
    }

    pub fn next_edge_id(&self) -> EdgeId {
        self.graph.next_edge_id()
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.graph.get_node(id)
    }

    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        self.graph.nodes()
    }
    pub fn node_order(&self) -> &Vec<NodeId> {
        self.graph.node_order()
    }

    pub fn new_edge(&self) -> Edge {
        self.graph.new_edge()
    }

    pub fn selection_bounds(&self) -> Option<Bounds<Pixels>> {
        self.graph.selection_bounds()
    }

    pub fn selected_nodes_with_positions(&self) -> HashMap<NodeId, Point<Pixels>> {
        self.graph.selected_nodes_with_positions()
    }

    pub fn hit_node(&self, mouse: Point<Pixels>) -> Option<NodeId> {
        self.graph.hit_node(mouse)
    }

    pub fn world_to_screen(&self, p: Point<Pixels>) -> Point<Pixels> {
        self.viewport.world_to_screen(p)
    }

    pub fn screen_to_world(&self, p: Point<Pixels>) -> Point<Pixels> {
        self.viewport.screen_to_world(p)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RenderLayer {
    Background,
    Edges,
    Nodes,
    Selection,
    Interaction,
    Overlay,
}

impl RenderLayer {
    pub const ALL: [RenderLayer; 6] = [
        RenderLayer::Background,
        RenderLayer::Edges,
        RenderLayer::Nodes,
        RenderLayer::Selection,
        RenderLayer::Interaction,
        RenderLayer::Overlay,
    ];
    pub fn index(self) -> usize {
        match self {
            RenderLayer::Background => 0,
            RenderLayer::Edges => 1,
            RenderLayer::Nodes => 2,
            RenderLayer::Selection => 3,
            RenderLayer::Interaction => 4,
            RenderLayer::Overlay => 5,
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
