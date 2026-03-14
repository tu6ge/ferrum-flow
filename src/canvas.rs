use gpui::{prelude::FluentBuilder, *};

use crate::{
    Edge, EdgeId, Node, NodeRenderContext, NodeRenderer, Port, PortId, PortKind,
    graph::Graph,
    plugin::{
        EventResult, FlowEvent, InitPluginContext, InputEvent, Plugin, PluginContext,
        PluginRegistry, RenderContext, RenderLayer,
    },
    renderer::RendererRegistry,
    viewport::Viewport,
};

mod edge;
mod types;
mod utils;
use edge::EdgeGeometry;
use types::*;
use utils::*;

pub use types::{InteractionHandler, InteractionResult, InteractionState};

pub const DEFAULT_NODE_WIDTH: Pixels = px(120.0);
pub const DEFAULT_NODE_HEIGHT: Pixels = px(60.0);

pub struct FlowCanvas {
    pub graph: Graph,
    drag_state: DragState,

    pub(crate) viewport: Viewport,

    pub(crate) registry: RendererRegistry,

    pub(crate) plugins_registry: PluginRegistry,

    pub(crate) focus_handle: FocusHandle,

    pub(crate) interaction: InteractionState,

    pub event_queue: Vec<FlowEvent>,
}

// TODO
impl Clone for FlowCanvas {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph.clone(),
            drag_state: self.drag_state.clone(),
            viewport: self.viewport.clone(),
            registry: self.registry.clone(),
            plugins_registry: PluginRegistry::new(),
            focus_handle: self.focus_handle.clone(),
            interaction: InteractionState::new(),
            event_queue: vec![],
        }
    }
}

impl FlowCanvas {
    pub fn new(graph: Graph, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            graph,
            drag_state: DragState::None,
            viewport: Viewport::new(),
            registry: RendererRegistry::new(),
            plugins_registry: PluginRegistry::new(),
            focus_handle,
            interaction: InteractionState::new(),
            event_queue: vec![],
        }
    }

    pub fn plugin(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugins_registry = self.plugins_registry.add(plugin);
        self
    }

    pub fn init_plugins(&mut self) {
        let mut ctx = InitPluginContext {
            graph: &mut self.graph,
            viewport: &mut self.viewport,
        };

        self.plugins_registry.plugins.sort_by_key(|p| -p.priority());

        for plugin in &mut self.plugins_registry.plugins.iter_mut() {
            plugin.setup(&mut ctx);
        }
    }

    pub fn handle_event(&mut self, event: FlowEvent, cx: &mut Context<Self>) {
        let event_queue = &mut self.event_queue;

        let mut emit = |event: FlowEvent| {
            event_queue.push(event);
        };

        let mut notify = || {
            cx.notify();
        };

        // if has interaction
        if let Some(mut handler) = self.interaction.handler.take() {
            let mut ctx = PluginContext::new(
                &mut self.graph,
                &mut self.viewport,
                &mut self.interaction,
                &mut emit,
                &mut notify,
            );
            let mut fast_return = false;
            let result = match &event {
                FlowEvent::Input(InputEvent::MouseMove(ev)) => {
                    fast_return = true;
                    handler.on_mouse_move(ev, &mut ctx)
                }

                FlowEvent::Input(InputEvent::MouseUp(ev)) => {
                    fast_return = true;
                    handler.on_mouse_up(ev, &mut ctx)
                }

                _ => InteractionResult::Continue,
            };

            if fast_return {
                match result {
                    InteractionResult::Continue => self.interaction.handler = Some(handler),

                    InteractionResult::End => {
                        self.interaction.handler = None;
                    }

                    InteractionResult::Replace(h) => {
                        self.interaction.handler = Some(h);
                    }
                }
                return;
            }
        }

        let mut ctx = PluginContext::new(
            &mut self.graph,
            &mut self.viewport,
            &mut self.interaction,
            &mut emit,
            &mut notify,
        );

        // 否则广播给 plugins
        for plugin in &mut self.plugins_registry.plugins {
            let result = plugin.on_event(&event, &mut ctx);
            match result {
                EventResult::Continue => {}
                EventResult::Stop => break,
            }
        }
    }

    fn process_event_queue(&mut self, cx: &mut Context<Self>) {
        while let Some(event) = self.event_queue.pop() {
            let mut emit = |e| self.event_queue.push(e);

            let mut notify = || {
                cx.notify();
            };

            let mut ctx = PluginContext::new(
                &mut self.graph,
                &mut self.viewport,
                &mut self.interaction,
                &mut emit,
                &mut notify,
            );

            for plugin in &mut self.plugins_registry.plugins {
                let result = plugin.on_event(&event, &mut ctx);
                match result {
                    EventResult::Continue => {}
                    EventResult::Stop => break,
                }
            }
        }
    }

    pub fn register_node<R>(mut self, name: impl Into<String>, renderer: R) -> Self
    where
        R: NodeRenderer + 'static,
    {
        self.registry.register(name, renderer);
        self
    }

    fn render_nodes(&self, _: &mut Context<Self>) -> Vec<impl IntoElement> {
        let nodes = self.graph.nodes();
        self.graph
            .node_order()
            .iter()
            .map(|node_id| {
                let node = nodes[node_id].clone();

                // custom node render
                if let Some(renderer) = self.registry.get(&node.node_type) {
                    let world_pos = Point::new(node.x, node.y);

                    let screen = self.viewport.world_to_screen(world_pos);

                    let size = renderer.size(&node);

                    let screen_w = size.width * self.viewport.zoom;

                    let screen_h = size.height * self.viewport.zoom;

                    let mut ctx = NodeRenderContext {
                        zoom: self.viewport.zoom,
                        rounded: px(5.0),
                    };

                    let inner = renderer.render(&node, &mut ctx);

                    let node_id_clone = node_id.clone();
                    let selected = self
                        .graph
                        .selected_node
                        .iter()
                        .find(|id| **id == node_id_clone)
                        .is_some();

                    div()
                        .absolute()
                        .left(screen.x)
                        .top(screen.y)
                        .w(screen_w)
                        .h(screen_h)
                        .rounded(px(6.0))
                        .border(px(1.5))
                        .when(selected, |div| div.border_color(rgb(0xFF7800)))
                        .child(div().absolute().size_full().child(inner))
                } else {
                    // default node render
                    let node_id = node.id;
                    let screen = self.viewport.world_to_screen(node.point());
                    let node_x = screen.x;
                    let node_y = screen.y;
                    let selected = self
                        .graph
                        .selected_node
                        .iter()
                        .find(|id| **id == node_id)
                        .is_some();

                    div()
                        .absolute()
                        .left(node_x)
                        .top(node_y)
                        .w(DEFAULT_NODE_WIDTH * self.viewport.zoom)
                        .h(DEFAULT_NODE_HEIGHT * self.viewport.zoom)
                        .bg(white())
                        .rounded(px(6.0))
                        .border(px(1.5))
                        .border_color(rgb(if selected { 0xFF7800 } else { 0x1A192B }))
                        .child(
                            div()
                                .child(format!("Node {}", node_id))
                                .text_color(rgb(0x1A192B)),
                        )
                }
            })
            .collect()
    }

    fn render_ports(&self, this_cx: &mut Context<Self>) -> Vec<impl IntoElement> {
        self.graph
            .ports
            .iter()
            .map(
                |(
                    _,
                    Port {
                        id, node_id, kind, ..
                    },
                )| {
                    let node_id_clone = node_id.clone();
                    let position = self.port_screen_position(*id);
                    let port_id_clone = id.clone();
                    let entity = this_cx.entity();
                    let entity_up = entity.clone();
                    let kind_clone = kind.clone();

                    div()
                        .absolute()
                        .left(position.x - px(6.0 * self.viewport.zoom))
                        .top(position.y - px(6.0 * self.viewport.zoom))
                        .w(px(12.0 * self.viewport.zoom))
                        .h(px(12.0 * self.viewport.zoom))
                        .rounded_full()
                        .bg(rgb(0x1A192B))
                        .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                            cx.stop_propagation();
                            cx.update_entity(&entity, |this, cx| {
                                this.drag_state = DragState::EdgeDrag(Connecting {
                                    node_id: node_id_clone,
                                    port_id: port_id_clone,
                                    mouse: event.position,
                                });
                                cx.notify();
                            });
                        })
                        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            cx.update_entity(&entity_up, |this, cx| {
                                if let DragState::EdgeDrag(connecting) = &this.drag_state {
                                    if connecting.node_id == node_id_clone {
                                        return;
                                    }
                                    let connecting_port = &this.graph.ports[&connecting.port_id];
                                    if connecting_port.kind.clone() == kind_clone {
                                        return;
                                    }

                                    let edge = this
                                        .graph
                                        .new_edge()
                                        .source(connecting.port_id.clone())
                                        .target(port_id_clone);

                                    this.graph.add_edge(edge);
                                    this.drag_state = DragState::None;
                                    cx.notify();
                                }
                            });
                        })
                },
            )
            .collect()
    }

    fn port_position(&self) -> Option<Point<Pixels>> {
        if let DragState::EdgeDrag(Connecting { port_id, .. }) = &self.drag_state {
            Some(self.port_screen_position(*port_id))
        } else {
            None
        }
    }
    fn port_offset(&self, node: &Node, port: &Port) -> Point<Pixels> {
        let node_size = if node.node_type.is_empty() {
            Size::new(DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT)
        } else {
            if let Some(render) = self.registry.get(&node.node_type) {
                render.size(node)
            } else {
                Size::new(DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT)
            }
        };

        match port.kind {
            PortKind::Input => Point::new(px(0.0), node_size.height / 2.0),

            PortKind::Output => Point::new(node_size.width, node_size.height / 2.0),
        }
    }

    fn port_screen_position(&self, port_id: PortId) -> Point<Pixels> {
        let port = &self.graph.ports[&port_id];
        let node = &self.graph.nodes()[&port.node_id];

        let node_pos = node.point();

        let offset = self.port_offset(node, port);

        self.viewport.world_to_screen(node_pos + offset)
    }
    fn render_connecting_edge(&self) -> impl IntoElement {
        if let DragState::EdgeDrag(connect) = &self.drag_state
            && let Some(start) = self.port_position()
        {
            let mouse: Point<Pixels> = connect.mouse;
            canvas(
                |_, _, _| {},
                move |_, _, win, _| {
                    if let Ok(line) = edge_bezier(start, mouse) {
                        win.paint_path(line, rgb(0xb1b1b8));
                    }
                },
            )
        } else {
            canvas(|_, _, _| {}, |_, _, _, _| {})
        }
    }
    fn render_edges(&self) -> impl IntoElement {
        let this = self.clone();
        canvas(
            |_, _, _| this,
            move |_, this, win, _| {
                for (_, edge) in this.graph.edges.iter() {
                    let geometry = this.edge_geometry(edge);

                    let selected = this
                        .graph
                        .selected_edge
                        .iter()
                        .find(|e| **e == edge.id)
                        .is_some();

                    let Some(EdgeGeometry { start, c1, c2, end }) = geometry else {
                        return;
                    };
                    let mut line = PathBuilder::stroke(px(1.0));
                    line.move_to(start);
                    line.cubic_bezier_to(end, c1, c2);

                    if let Ok(line) = line.build() {
                        win.paint_path(line, rgb(if selected { 0xFF7800 } else { 0xb1b1b8 }));
                    }
                }
            },
        )
    }

    fn edge_geometry(&self, edge: &Edge) -> Option<EdgeGeometry> {
        let Edge {
            source_port,
            target_port,
            ..
        } = edge;

        let start = self.port_screen_position(*source_port);
        let end = self.port_screen_position(*target_port);

        Some(EdgeGeometry {
            start,
            c1: start + Point::new(px(50.0), px(0.0)),
            c2: end - Point::new(px(50.0), px(0.0)),
            end,
        })
    }

    fn hit_test_edge(&self, mouse: Point<Pixels>, edge: &Edge) -> bool {
        let Some(geom) = self.edge_geometry(edge) else {
            return false;
        };

        let points = sample_bezier(&geom, 20);

        for segment in points.windows(2) {
            let d = distance_to_segment(mouse, segment[0], segment[1]);

            if d < 8.0 {
                return true;
            }
        }

        false
    }

    fn hit_test_get_edge(&self, mouse: Point<Pixels>) -> Option<EdgeId> {
        for edge in self.graph.edges.values() {
            let Some(geom) = self.edge_geometry(edge) else {
                continue;
            };

            let bound = edge_bounds(&geom);
            if !bound.contains(&mouse) {
                continue;
            }

            if self.hit_test_edge(mouse, edge) {
                return Some(edge.id);
            }
        }

        None
    }

    fn on_key_down(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if ev.keystroke.key == "delete" || ev.keystroke.key == "backspace" {
            if self.graph.remove_selected_edge() {
                cx.notify();
            }
            if self.graph.remove_selected_node() {
                cx.notify();
            }
        }
    }

    fn on_mouse_down(&mut self, ev: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::MouseDown(ev.clone())), cx);
        self.process_event_queue(cx);
        let shift = ev.modifiers.shift;

        if let Some(id) = self.hit_test_get_edge(ev.position) {
            self.graph.add_selected_edge(id, shift);
        } else {
            if !shift {
                self.graph.clear_selected_edge();
            }
        }

        cx.notify();
    }

    fn on_mouse_move(&mut self, ev: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::MouseMove(ev.clone())), cx);
        self.process_event_queue(cx);
        match &mut self.drag_state {
            DragState::EdgeDrag(connect) => {
                connect.mouse = ev.position;
                cx.notify();
            }
            _ => (),
        }
    }

    fn on_mouse_up(&mut self, ev: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::MouseUp(ev.clone())), cx);
        self.process_event_queue(cx);
        match &self.drag_state {
            DragState::EdgeDrag(_) => {
                self.drag_state = DragState::None;
                cx.notify();
            }
            _ => (),
        };
    }

    fn on_scroll_wheel(&mut self, ev: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::Wheel(ev.clone())), cx);
        self.process_event_queue(cx);
    }
}

fn edge_bezier(start: Point<Pixels>, end: Point<Pixels>) -> Result<Path<Pixels>, anyhow::Error> {
    let mut line = PathBuilder::stroke(px(1.0));
    line.move_to(start);
    line.cubic_bezier_to(
        end,
        Point::new(start.x + px(50.0), start.y),
        Point::new(end.x - px(50.0), end.y),
    );

    line.build()
}

fn sample_bezier(geom: &EdgeGeometry, steps: usize) -> Vec<Point<Pixels>> {
    let mut points = Vec::new();

    for i in 0..=steps {
        let t = i as f32 / steps as f32;

        let x = (1.0 - t).powi(3) * geom.start.x
            + 3.0 * (1.0 - t).powi(2) * t * geom.c1.x
            + 3.0 * (1.0 - t) * t * t * geom.c2.x
            + t.powi(3) * geom.end.x;

        let y = (1.0 - t).powi(3) * geom.start.y
            + 3.0 * (1.0 - t).powi(2) * t * geom.c1.y
            + 3.0 * (1.0 - t) * t * t * geom.c2.y
            + t.powi(3) * geom.end.y;

        points.push(Point::new(x, y));
    }

    points
}

impl Render for FlowCanvas {
    fn render(&mut self, window: &mut Window, this_cx: &mut Context<Self>) -> impl IntoElement {
        let entity = this_cx.entity();

        let graph = &self.graph;
        let viewport = &self.viewport;

        let mut layers: Vec<Vec<AnyElement>> =
            (0..RenderLayer::ALL.len()).map(|_| Vec::new()).collect();

        for plugin in self.plugins_registry.plugins.iter_mut() {
            let layer = plugin.render_layer();

            let mut ctx = RenderContext::new(graph, viewport, window, layer);

            if let Some(el) = plugin.render(&mut ctx) {
                layers[layer.index()].push(el);
            }
        }

        if let Some(i) = self.interaction.handler.as_ref() {
            let mut ctx = RenderContext::new(graph, viewport, window, RenderLayer::Interaction);

            if let Some(el) = i.render(&mut ctx) {
                layers[RenderLayer::Interaction.index()].push(el);
            }
        }

        div()
            .size_full()
            .track_focus(&self.focus_handle)
            // bg point 9F9FA7
            .bg(gpui::rgb(0xf8f9fb))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&entity, Self::on_mouse_down),
            )
            .on_key_down(window.listener_for(&entity, Self::on_key_down))
            .on_mouse_move(window.listener_for(&entity, Self::on_mouse_move))
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&entity, Self::on_mouse_up),
            )
            .on_scroll_wheel(window.listener_for(&entity, Self::on_scroll_wheel))
            .child(self.render_connecting_edge())
            .child(self.render_edges())
            .children(self.render_nodes(this_cx))
            .children(self.render_ports(this_cx))
            .children(
                RenderLayer::ALL
                    .iter()
                    .map(|layer| div().absolute().children(layers[layer.index()].drain(..))),
            )
    }
}
