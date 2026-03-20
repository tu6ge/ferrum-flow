use std::collections::HashMap;

use gpui::{Bounds, Pixels, Point};

use crate::{
    Edge, EdgeId, Graph, Node, NodeBuilder, NodeId, Port, PortId, RendererRegistry, Viewport,
    canvas::PortLayoutCache,
};

pub trait Command {
    fn name(&self) -> &'static str;
    fn execute(&mut self, ctx: &mut CommandContext);
    fn undo(&mut self, ctx: &mut CommandContext);
}

pub struct CommandContext<'a> {
    pub graph: &'a mut Graph,
    pub port_offset_cache: &'a mut PortLayoutCache,
    pub viewport: &'a mut Viewport,
    pub renderers: &'a mut RendererRegistry,
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

    pub fn execute(&mut self, mut command: Box<dyn Command>, state: &mut CommandContext) {
        command.execute(state);

        self.undo_stack.push(command);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, state: &mut CommandContext) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(state);
            self.redo_stack.push(cmd);
        }
    }

    pub fn redo(&mut self, state: &mut CommandContext) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(state);
            self.undo_stack.push(cmd);
        }
    }
}

pub struct CompositeCommand {
    commands: Vec<Box<dyn Command>>,
}

impl CompositeCommand {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
    pub fn push(&mut self, command: impl Command + 'static) {
        self.commands.push(Box::new(command));
    }
}

impl Command for CompositeCommand {
    fn name(&self) -> &'static str {
        "composite"
    }
    fn execute(&mut self, state: &mut CommandContext) {
        for cmd in &mut self.commands {
            cmd.execute(state);
        }
    }

    fn undo(&mut self, state: &mut CommandContext) {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(state);
        }
    }
}

impl<'a> CommandContext<'a> {
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
        self.port_offset_cache.clear_node(id);
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

    pub fn is_node_visible(&self, node_id: &NodeId) -> bool {
        let Some(node) = self.get_node(node_id) else {
            return false;
        };
        self.viewport.is_node_visible(node)
    }

    pub fn is_edge_visible(&self, edge: &Edge) -> bool {
        let Edge {
            source_port,
            target_port,
            ..
        } = edge;
        let Some(Port { node_id, .. }) = self.graph.ports.get(source_port) else {
            return false;
        };
        let Some(Port {
            node_id: node_id2, ..
        }) = self.graph.ports.get(target_port)
        else {
            return false;
        };
        self.is_node_visible(node_id) || self.is_node_visible(node_id2)
    }

    pub fn port_offset_cached(&self, node_id: &NodeId, port_id: &PortId) -> Option<Point<Pixels>> {
        if let Some(node_cache) = self.port_offset_cache.map.get(node_id) {
            if let Some(pos) = node_cache.get(port_id) {
                return Some(*pos);
            }
        }

        None
    }

    pub fn cache_all_node_port_offset(&mut self) {
        let nodes: Vec<NodeId> = self.graph.nodes().iter().map(|(id, _)| *id).collect();

        for node in nodes {
            self.cache_node_port_offset(&node);
        }
    }

    fn cache_node_port_offset(&mut self, node_id: &NodeId) {
        if self.port_offset_cache.map.get(&node_id).is_some() {
            return;
        }
        let Some(node) = self.get_node(node_id) else {
            return;
        };
        let renderer = self.renderers.get(&node.node_type);

        let mut result = HashMap::new();

        for port in self.graph.ports.values().filter(|p| p.node_id == node.id) {
            let pos = renderer.port_offset(node, port, self.graph);
            result.insert(port.id, pos);
        }

        self.port_offset_cache.map.insert(node.id, result);
    }
}
