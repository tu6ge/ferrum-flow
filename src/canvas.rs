use std::collections::HashMap;

use gpui::{prelude::FluentBuilder, *};

use crate::{
    Edge, EdgeId, Node, NodeId, NodeRenderContext, NodeRenderer, Port, PortId, PortKind,
    graph::Graph, renderer::RendererRegistry, viewport::Viewport,
};

mod edge;
mod types;
mod utils;
use edge::EdgeGeometry;
use types::*;
use utils::*;

const DEFAULT_NODE_WIDTH: Pixels = px(120.0);
const DEFAULT_NODE_HEIGHT: Pixels = px(60.0);
const DRAG_THRESHOLD: Pixels = px(2.0);

#[derive(Clone)]
pub struct FlowCanvas {
    pub graph: Graph,
    drag_state: DragState,

    viewport: Viewport,

    registry: RendererRegistry,

    focus_handle: FocusHandle,

    box_selection: Option<BoxSelection>,
}

impl FlowCanvas {
    pub fn new(graph: Graph, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            graph,
            drag_state: DragState::None,
            viewport: Viewport::new(),
            registry: RendererRegistry::new(),
            focus_handle,
            box_selection: None,
        }
    }

    pub fn register_node<R>(mut self, name: impl Into<String>, renderer: R) -> Self
    where
        R: NodeRenderer + 'static,
    {
        self.registry.register(name, renderer);
        self
    }

    fn render_nodes(&self, this_cx: &mut Context<Self>) -> Vec<impl IntoElement> {
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

                    let entry = this_cx.entity();
                    let this_entity_move = entry.clone();
                    let this_entity_up = entry.clone();
                    let node_id_clone = node_id.clone();
                    let node_point = node.point();
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
                        .on_mouse_down(MouseButton::Left, move |ev, _win, cx| {
                            cx.stop_propagation();

                            cx.update_entity(&entry, move |this: &mut Self, cx| {
                                this.drag_state = DragState::PendingNode(PendingNode {
                                    node_id: node_id_clone,
                                    start_mouse: ev.position,
                                    shift: ev.modifiers.shift,
                                });
                                cx.notify();
                            });
                        })
                        .on_mouse_move(move |ev, _, cx| {
                            cx.stop_propagation();
                            cx.update_entity(&this_entity_move, |this, cx| match &this.drag_state {
                                DragState::NodeDrag(NodeDrag {
                                    start_mouse,
                                    start_positions,
                                }) => {
                                    let dx = (ev.position.x - start_mouse.x) / this.viewport.zoom;
                                    let dy = (ev.position.y - start_mouse.y) / this.viewport.zoom;
                                    for (id, point) in start_positions.iter() {
                                        if let Some(node) = this.graph.get_node_mut(*id) {
                                            node.x = point.x + dx;
                                            node.y = point.y + dy;
                                        }
                                    }
                                    cx.notify();
                                }
                                DragState::PendingNode(PendingNode {
                                    node_id: id,
                                    start_mouse,
                                    ..
                                }) if node_id_clone == *id => {
                                    let delta = ev.position - *start_mouse;
                                    if delta.x > DRAG_THRESHOLD || delta.y > DRAG_THRESHOLD {
                                        this.start_node_drag(
                                            ev.position,
                                            node_id_clone,
                                            node_point,
                                        );
                                        cx.notify();
                                    }
                                }
                                _ => {}
                            })
                        })
                        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            cx.update_entity(&this_entity_up, |this, cx| {
                                if let DragState::PendingNode(PendingNode {
                                    node_id: id,
                                    shift,
                                    ..
                                }) = this.drag_state
                                    && id == node_id_clone
                                {
                                    if !shift {
                                        this.graph.clear_selected_edge();
                                    }
                                    this.graph.add_selected_node(id, shift);
                                    this.bring_node_to_front(node_id_clone);
                                }

                                this.drag_state = DragState::None;
                                cx.notify();
                            })
                        })
                        .rounded(px(6.0))
                        .border(px(1.5))
                        .when(selected, |div| div.border_color(rgb(0xFF7800)))
                        .child(div().absolute().size_full().child(inner))
                } else {
                    // default node render
                    let entry = this_cx.entity();
                    let this_entity_move = entry.clone();
                    let this_entity_up = entry.clone();
                    let node_id = node.id;
                    let node_point = node.point();
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
                        .on_mouse_down(MouseButton::Left, move |ev, _win, cx| {
                            cx.stop_propagation();

                            cx.update_entity(&entry, |this: &mut Self, cx| {
                                this.drag_state = DragState::PendingNode(PendingNode {
                                    node_id: node_id,
                                    start_mouse: ev.position,
                                    shift: ev.modifiers.shift,
                                });
                                cx.notify();
                            });
                        })
                        .on_mouse_move(move |ev, _, cx| {
                            cx.stop_propagation();
                            cx.update_entity(&this_entity_move, |this, cx| match &this.drag_state {
                                DragState::NodeDrag(NodeDrag {
                                    start_mouse,
                                    start_positions,
                                }) => {
                                    let dx = (ev.position.x - start_mouse.x) / this.viewport.zoom;
                                    let dy = (ev.position.y - start_mouse.y) / this.viewport.zoom;
                                    for (id, point) in start_positions.iter() {
                                        if let Some(node) = this.graph.get_node_mut(*id) {
                                            node.x = point.x + dx;
                                            node.y = point.y + dy;
                                        }
                                    }
                                    cx.notify();
                                }
                                DragState::PendingNode(PendingNode {
                                    node_id: id,
                                    start_mouse,
                                    ..
                                }) if node_id == *id => {
                                    let delta = ev.position - *start_mouse;
                                    if delta.x.abs() > DRAG_THRESHOLD
                                        || delta.y.abs() > DRAG_THRESHOLD
                                    {
                                        this.start_node_drag(ev.position, node_id, node_point);
                                        cx.notify();
                                    }
                                }
                                _ => {}
                            })
                        })
                        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            cx.update_entity(&this_entity_up, |this, cx| {
                                if let DragState::PendingNode(PendingNode {
                                    node_id: id,
                                    shift,
                                    ..
                                }) = this.drag_state
                                    && id == node_id
                                {
                                    if !shift {
                                        this.graph.clear_selected_edge();
                                    }
                                    this.graph.add_selected_node(id, shift);
                                    this.bring_node_to_front(node_id);
                                }

                                this.drag_state = DragState::None;
                                cx.notify();
                            })
                        })
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

    fn start_node_drag(&mut self, mouse: Point<Pixels>, node_id: NodeId, point: Point<Pixels>) {
        let start_positions = if self.graph.selected_node.contains(&node_id) {
            self.graph
                .selected_node
                .iter()
                .map(|id| (*id, self.graph.nodes()[id].point()))
                .collect()
        } else {
            vec![(node_id, point)]
        };

        let drag = NodeDrag {
            start_mouse: mouse,
            start_positions,
        };
        self.drag_state = DragState::NodeDrag(drag);
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

    fn render_grid(&self, win: &mut Window) -> impl IntoElement {
        let base_grid = 40.0;
        let zoom = self.viewport.zoom;

        let grid = base_grid * zoom;

        let offset = self.viewport.offset;

        let start_x = f32::from(offset.x) % grid;
        let start_y = f32::from(offset.y) % grid;

        let mut dots = Vec::new();

        let bounds = win.bounds();
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);

        let mut x = start_x;

        while x < width {
            let mut y = start_y;

            while y < height {
                dots.push(
                    div()
                        .absolute()
                        .left(px(x))
                        .top(px(y))
                        .w(px(2.0))
                        .h(px(2.0))
                        .rounded_full()
                        .bg(rgb(0x9F9FA7)),
                );

                y += grid;
            }

            x += grid;
        }

        div().absolute().size_full().children(dots)
    }

    fn node_screen_bounds(&self, node: &Node) -> Bounds<Pixels> {
        let pos = self.viewport.world_to_screen(Point::new(node.x, node.y));

        let w = DEFAULT_NODE_WIDTH * self.viewport.zoom;
        let h = DEFAULT_NODE_HEIGHT * self.viewport.zoom;

        Bounds::new(pos, Size::new(w, h))
    }

    fn hit_test_node(&self, mouse: Point<Pixels>) -> Option<NodeId> {
        let nodes = self.graph.nodes();
        for id in self.graph.node_order().iter().rev() {
            let node = &nodes[id];
            let bounds = self.node_screen_bounds(&node);

            if bounds.contains(&mouse) {
                return Some(node.id);
            }
        }
        None
    }

    fn bring_node_to_front(&mut self, node_id: NodeId) {
        self.graph.node_order_mut().retain(|id| *id != node_id);

        self.graph.node_order_mut().push(node_id);
    }

    fn selection_bounds_mouse(&self) -> Option<(Bounds<Pixels>, Point<Pixels>)> {
        match self.drag_state {
            DragState::BoxSelect(BoxSelectDrag { start, end, .. }) => {
                let size = Size::new(end.x - start.x, end.y - start.y);
                Some((Bounds::new(start, size), start))
            }
            _ => None,
        }
    }

    fn finalize_selection(&mut self) {
        let rect = self.selection_bounds_mouse();

        let Some((rect, mouse)) = rect else {
            return;
        };

        self.graph.clear_selected_node();

        if rect.size.width < px(4.0) || rect.size.height < px(4.0) {
            self.drag_state = DragState::None;
            return;
        }

        let mut selected_ids = HashMap::new();

        for node in self.graph.nodes().values() {
            let pos = self.node_screen_bounds(node);

            if rect.intersects(&pos) {
                selected_ids.insert(node.id, node.point());
            }
        }

        for (id, _) in selected_ids.iter() {
            self.graph.add_selected_node(*id, true);
        }

        self.box_selection = Some(BoxSelection {
            start_mouse: mouse,
            bounds: rect,
            nodes: selected_ids,
        });
        self.drag_state = DragState::None;
    }

    fn render_draging_select_box(&self) -> impl IntoElement {
        let DragState::BoxSelect(BoxSelectDrag { start, end, .. }) = &self.drag_state else {
            return div();
        };
        div()
            .absolute()
            .left(start.x)
            .top(start.y)
            .w(end.x - start.x)
            .h(end.y - start.y)
            .border(px(1.0))
            .border_color(rgb(0x78A0FF))
            .bg(rgba(0x78A0FF4c))
    }

    fn render_selected_box(
        &self,
        window: &mut Window,
        this_cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(BoxSelection { bounds, .. }) = &self.box_selection else {
            return div();
        };
        let this_entity = this_cx.entity();
        div()
            .absolute()
            .left(bounds.origin.x)
            .top(bounds.origin.y)
            .w(bounds.size.width)
            .h(bounds.size.height)
            .border(px(1.0))
            .border_color(rgb(0x78A0FF))
            .bg(rgba(0x78A0FF4c))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&this_entity, Self::box_on_mouse_down),
            )
            .on_mouse_move(window.listener_for(&this_entity, Self::box_on_mouse_move))
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&this_entity, Self::box_on_mouse_up),
            )
    }

    fn box_on_mouse_down(&mut self, ev: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        if let Some(BoxSelection { bounds, nodes, .. }) = &self.box_selection {
            self.drag_state = DragState::BoxMove(BoxMoveDrag {
                start_mouse: ev.position,
                start_bounds: *bounds,
                nodes: nodes.iter().map(|(k, v)| (*k, *v)).collect(),
            });

            cx.notify();
        }
    }

    fn box_on_mouse_move(&mut self, ev: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        let DragState::BoxMove(BoxMoveDrag {
            //start_bounds,
            nodes,
            start_mouse,
            start_bounds,
            ..
        }) = &mut self.drag_state
        else {
            return;
        };
        let Some(BoxSelection {
            bounds,
            nodes: box_nodes,
            ..
        }) = &mut self.box_selection
        else {
            return;
        };
        {
            let dx = (ev.position.x - start_mouse.x) / self.viewport.zoom;
            let dy = (ev.position.y - start_mouse.y) / self.viewport.zoom;
            for (id, point) in nodes.iter() {
                if let Some(node) = self.graph.get_node_mut(*id) {
                    node.x = point.x + dx;
                    node.y = point.y + dy;
                }
                if let Some(node) = box_nodes.get_mut(id) {
                    node.x = point.x + dx;
                    node.y = point.y + dy;
                }
            }
            //let box_bounds = Bounds::new(*box_start_mouse, bounds.size);

            let dx = ev.position.x - start_mouse.x;
            let dy = ev.position.y - start_mouse.y;
            bounds.origin.x = start_bounds.origin.x + dx;
            bounds.origin.y = start_bounds.origin.y + dy;

            cx.notify();
        }
    }

    fn box_on_mouse_up(&mut self, ev: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        if let DragState::BoxMove(_) = self.drag_state {
            self.drag_state = DragState::None;
            let Some(BoxSelection {
                start_mouse: box_start_mouse,
                ..
            }) = &mut self.box_selection
            else {
                return;
            };
            *box_start_mouse = ev.position;
            cx.notify();
        }
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
        let shift = ev.modifiers.shift;

        let mut selected_edge = false;

        if let Some(id) = self.hit_test_get_edge(ev.position) {
            self.graph.add_selected_edge(id, shift);
            selected_edge = true;
        } else {
            if !shift {
                self.graph.clear_selected_edge();
            }
        }

        if let Some(id) = self.hit_test_node(ev.position) {
            self.graph.add_selected_node(id, shift);
        } else {
            if !shift {
                self.graph.clear_selected_node();
            }
        }

        if shift {
            self.drag_state = DragState::Pan(Panning {
                start_mouse: ev.position,
                start_offset: self.viewport.offset,
            });
        }

        if !shift && !selected_edge {
            self.drag_state = DragState::PendingBoxSelect(PendingBoxSelect { start: ev.position });
            self.box_selection = None;
        }
        if shift && let DragState::BoxSelect(_) = self.drag_state {
            self.drag_state = DragState::None;
        }
        cx.notify();
    }

    fn on_mouse_move(&mut self, ev: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        match &mut self.drag_state {
            DragState::EdgeDrag(connect) => {
                connect.mouse = ev.position;
                cx.notify();
            }
            DragState::Pan(Panning {
                start_mouse,
                start_offset,
            }) => {
                let dx = ev.position.x - start_mouse.x;
                let dy = ev.position.y - start_mouse.y;

                self.viewport.offset.x = start_offset.x + dx;
                self.viewport.offset.y = start_offset.y + dy;
                cx.notify();
            }
            DragState::PendingBoxSelect(pending) => {
                let delta = ev.position - pending.start;
                if delta.x > DRAG_THRESHOLD || delta.y > DRAG_THRESHOLD {
                    self.drag_state = DragState::BoxSelect(BoxSelectDrag {
                        start: pending.start,
                        end: ev.position,
                    })
                }
            }
            DragState::BoxSelect(selection_box) => {
                selection_box.end = ev.position;
                cx.notify();
            }
            _ => (),
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        match &self.drag_state {
            DragState::Pan(_)
            | DragState::NodeDrag(_)
            | DragState::PendingNode(_)
            | DragState::EdgeDrag(_)
            | DragState::PendingBoxSelect(_) => {
                self.drag_state = DragState::None;
                cx.notify();
            }
            DragState::BoxSelect(_) => {
                self.finalize_selection();
                cx.notify();
            }
            _ => (),
        };
    }

    fn on_scroll_wheel(&mut self, ev: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let cursor = ev.position;

        let before = self.viewport.screen_to_world(cursor);

        let delta = f32::from(ev.delta.pixel_delta(px(1.0)).y);
        if delta == 0.0 {
            return;
        }

        self.drag_state = DragState::None;
        self.box_selection = None;

        let zoom_delta = if delta > 0.0 { 0.9 } else { 1.1 };

        self.viewport.zoom *= zoom_delta;

        self.viewport.zoom = self.viewport.zoom.clamp(0.7, 3.0);

        let after = self.viewport.world_to_screen(before);

        self.viewport.offset.x += cursor.x - after.x;
        self.viewport.offset.y += cursor.y - after.y;

        cx.notify();
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
            .child(self.render_grid(window))
            .child(self.render_connecting_edge())
            .child(self.render_edges())
            .children(self.render_nodes(this_cx))
            .children(self.render_ports(this_cx))
            .child(self.render_draging_select_box())
            .child(self.render_selected_box(window, this_cx))
    }
}
