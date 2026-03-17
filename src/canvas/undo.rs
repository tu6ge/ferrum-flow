use std::collections::HashMap;

use gpui::{Bounds, Pixels, Point};

use crate::{Edge, EdgeId, Graph, Node, NodeId, Port, Viewport};

pub trait Command {
    fn name(&self) -> &'static str;
    fn execute(&mut self, ctx: &mut CanvasState);
    fn undo(&mut self, ctx: &mut CanvasState);
}

pub struct CanvasState<'a> {
    pub graph: &'a mut Graph,
    pub viewport: &'a mut Viewport,
}

const MAX_HISTORY: usize = 100;

pub struct History {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, mut command: Box<dyn Command>, state: &mut CanvasState) {
        command.execute(state);

        self.undo_stack.push(command);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, state: &mut CanvasState) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(state);
            self.redo_stack.push(cmd);
        }
    }

    pub fn redo(&mut self, state: &mut CanvasState) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(state);
            self.undo_stack.push(cmd);
        }
    }
}

pub struct CompositeCommand {
    commands: Vec<Box<dyn Command>>,
}

impl Command for CompositeCommand {
    fn name(&self) -> &'static str {
        "composite"
    }
    fn execute(&mut self, state: &mut CanvasState) {
        for cmd in &mut self.commands {
            cmd.execute(state);
        }
    }

    fn undo(&mut self, state: &mut CanvasState) {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(state);
        }
    }
}

impl<'a> CanvasState<'a> {
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
