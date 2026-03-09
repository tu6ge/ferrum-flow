use gpui::{prelude::FluentBuilder, *};

use crate::{
    Edge, EdgeId, Node, NodeId, NodeRenderContext, NodeRenderer, graph::Graph,
    renderer::RendererRegistry, viewport::Viewport,
};

mod edge;
mod utils;
use edge::EdgeGeometry;
use utils::*;

const DEFAULT_NODE_WIDTH: Pixels = px(120.0);
const DEFAULT_NODE_HEIGHT: Pixels = px(60.0);

#[derive(Clone)]
pub struct FlowCanvas {
    pub graph: Graph,
    dragging_node: Option<DraggingNode>,
    connecting: Option<Connecting>,

    viewport: Viewport,
    panning: Option<Panning>,

    registry: RendererRegistry,

    focus_handle: FocusHandle,
}

#[derive(Debug, Clone)]
struct DraggingNode {
    node_id: NodeId,
    start_mouse: Point<Pixels>,
    start_node: Point<Pixels>,
}

#[derive(Debug, Clone)]
struct Connecting {
    node_id: NodeId,
    port_id: String,
    mouse: Point<Pixels>,
}

#[derive(Debug, Clone)]
struct Panning {
    start_mouse: Point<Pixels>,
    start_offset: Point<Pixels>,
}

impl FlowCanvas {
    pub fn new(graph: Graph, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            graph,
            dragging_node: None,
            connecting: None,
            viewport: Viewport::new(),
            panning: None,
            registry: RendererRegistry::new(),
            focus_handle,
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
                    let node_point = node.point();
                    let node_id_clone = node_id.clone();
                    let selected = self.graph.selected_node == Some(node_id_clone);

                    div()
                        .absolute()
                        .left(screen.x)
                        .top(screen.y)
                        .w(screen_w)
                        .h(screen_h)
                        .on_mouse_down(MouseButton::Left, move |ev, _win, cx| {
                            cx.stop_propagation();

                            cx.update_entity(&entry, |this: &mut Self, cx| {
                                this.dragging_node = Some(DraggingNode {
                                    node_id: node_id_clone,
                                    start_mouse: ev.position,
                                    start_node: node_point,
                                });
                                this.graph.selected_edge = None;
                                this.graph.selected_node = Some(node_id_clone.clone());
                                this.bring_node_to_front(node_id_clone.clone());
                                cx.notify();
                            });
                        })
                        .rounded(px(6.0))
                        .border(px(1.5))
                        .when(selected, |div| div.border_color(rgb(0xFF7800)))
                        .child(div().absolute().size_full().child(inner))
                        .child(self.render_ports(&node, this_cx))
                } else {
                    // default node render
                    let entry = this_cx.entity();
                    let node_id = node.id;
                    let node_point = node.point();
                    let screen = self.viewport.world_to_screen(node.point());
                    let node_x = screen.x;
                    let node_y = screen.y;
                    let selected = self.graph.selected_node == Some(node_id);

                    div()
                        .absolute()
                        .left(node_x)
                        .top(node_y)
                        .on_mouse_down(MouseButton::Left, move |ev, _win, cx| {
                            cx.stop_propagation();

                            cx.update_entity(&entry, |this: &mut Self, cx| {
                                this.dragging_node = Some(DraggingNode {
                                    node_id: node_id.clone(),
                                    start_mouse: ev.position,
                                    start_node: node_point,
                                });
                                this.graph.selected_edge = None;
                                this.graph.selected_node = Some(node_id.clone());
                                this.bring_node_to_front(node_id.clone());
                                cx.notify();
                            });
                        })
                        .w(DEFAULT_NODE_WIDTH * self.viewport.zoom)
                        .h(DEFAULT_NODE_HEIGHT * self.viewport.zoom)
                        .bg(white())
                        .rounded(px(6.0))
                        .border(px(1.5))
                        .border_color(rgb(if selected { 0xFF7800 } else { 0x1A192B }))
                        .child(self.render_ports(&node, this_cx))
                        .child(
                            div()
                                .child(format!("Node {}", node_id))
                                .text_color(rgb(0x1A192B)),
                        )
                }
            })
            .collect()
    }

    fn render_ports(&self, node: &Node, this_cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .size_full()
            .children(node.outputs.iter().map(|port| {
                let entity = this_cx.entity();
                let node_id = node.id;
                let port_id = port.id.clone();
                div()
                    .absolute()
                    .left((port.point.x - px(8.0)) * self.viewport.zoom)
                    .top((port.point.y - px(8.0)) * self.viewport.zoom)
                    .w(px(12.0 * self.viewport.zoom))
                    .h(px(12.0 * self.viewport.zoom))
                    .rounded_full()
                    .bg(rgb(0x1A192B))
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        cx.stop_propagation();
                        cx.update_entity(&entity, |this, cx| {
                            this.connecting = Some(Connecting {
                                node_id,
                                port_id: port_id.clone(),
                                mouse: event.position.clone(),
                            });
                            cx.notify();
                        });
                    })
            }))
            .children(node.inputs.iter().map(|port| {
                let entity = this_cx.entity();
                let node_id = node.id;
                let port_id = port.id.clone();
                div()
                    .absolute()
                    .left((port.point.x - px(7.0)) * self.viewport.zoom)
                    .top((port.point.y - px(7.0)) * self.viewport.zoom)
                    .w(px(12.0 * self.viewport.zoom))
                    .h(px(12.0 * self.viewport.zoom))
                    .rounded_full()
                    .bg(rgb(0x1A192B))
                    .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                        cx.update_entity(&entity, |this, cx| {
                            if let Some(connecting) = &this.connecting {
                                let edge = this
                                    .graph
                                    .new_edge()
                                    .source(connecting.node_id, connecting.port_id.clone())
                                    .target(node_id, port_id.clone());

                                this.graph.add_edge(edge);
                                this.connecting = None;
                                cx.notify();
                            }
                        });
                    })
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
            }))
    }

    fn port_position(&self) -> Option<Point<Pixels>> {
        if let Some(Connecting {
            node_id, port_id, ..
        }) = &self.connecting
        {
            self.graph
                .get_node(&node_id)
                .map(|node| {
                    node.outputs
                        .iter()
                        .find(|port| *port.id == *port_id)
                        .map(|port| (node, port))
                })
                .flatten()
                .map(|(node, port)| {
                    self.viewport
                        .world_to_screen(Point::new(node.x + port.point.x, node.y + port.point.y))
                })
        } else {
            None
        }
    }
    fn render_connecting_edge(&self) -> impl IntoElement {
        if let Some(connect) = &self.connecting
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

                    let selected = this.graph.selected_edge == Some(edge.id);

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
            source_node,
            target_node,
            source_port,
            target_port,
            ..
        } = edge;
        let Some(source_node) = self.graph.get_node(&source_node) else {
            return None;
        };
        let Some(target_node) = self.graph.get_node(&target_node) else {
            return None;
        };
        let source_point = source_node
            .outputs
            .iter()
            .find(|p| p.id == source_port.clone())
            .map(|p| p.point);
        let Some(source_point) = source_point else {
            return None;
        };
        let target_point = target_node
            .inputs
            .iter()
            .find(|p| p.id == target_port.clone())
            .map(|p| p.point);
        let Some(target_point) = target_point else {
            return None;
        };

        Some(EdgeGeometry {
            start: self.viewport.world_to_screen(Point::new(
                source_node.x + source_point.x,
                source_node.y + source_point.y,
            )),
            c1: self.viewport.world_to_screen(Point::new(
                source_node.x + source_point.x,
                source_node.y + source_point.y + px(50.0),
            )),
            c2: self.viewport.world_to_screen(Point::new(
                target_node.x + target_point.x,
                target_node.y + target_point.y - px(50.0),
            )),
            end: self.viewport.world_to_screen(Point::new(
                target_node.x + target_point.x,
                target_node.y + target_point.y,
            )),
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
}

fn edge_bezier(start: Point<Pixels>, end: Point<Pixels>) -> Result<Path<Pixels>, anyhow::Error> {
    let mut line = PathBuilder::stroke(px(1.0));
    line.move_to(start);
    line.cubic_bezier_to(
        end,
        Point::new(start.x, start.y + px(50.0)),
        Point::new(end.x, end.y - px(50.0)),
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
        let entry = this_cx.entity();
        let entry2 = entry.clone();
        let entity3 = entry.clone();
        let entity_mouse_down: Entity<FlowCanvas> = entry.clone();
        let entity_key_down = entry.clone();

        div()
            .size_full()
            .track_focus(&self.focus_handle)
            // bg point 9F9FA7
            .bg(gpui::rgb(0xf8f9fb))
            .on_mouse_down(MouseButton::Left, move |ev, _, app| {
                app.update_entity(&entity_mouse_down, |this, cx| {
                    this.panning = Some(Panning {
                        start_mouse: ev.position,
                        start_offset: this.viewport.offset,
                    });

                    this.graph.selected_edge = this.hit_test_get_edge(ev.position);

                    this.graph.selected_node = this.hit_test_node(ev.position);

                    cx.notify();
                })
            })
            .on_key_down(move |ev, _, app| {
                app.update_entity(&entity_key_down, |this, cx| {
                    if ev.keystroke.key == "delete" || ev.keystroke.key == "backspace" {
                        if let Some(edge_id) = this.graph.selected_edge.take() {
                            this.graph.edges.remove(&edge_id);
                            cx.notify();
                        } else if let Some(node_id) = this.graph.selected_node.take() {
                            this.graph.remove_node(&node_id);
                            cx.notify();
                        }
                    }
                })
            })
            .on_mouse_move(move |ev, _, cx| {
                //println!("mouse move");
                cx.update_entity(&entry, |this, cx| {
                    if let Some(connect) = &mut this.connecting {
                        connect.mouse = ev.position;
                        cx.notify();
                    } else if let Some(DraggingNode {
                        node_id,
                        start_mouse,
                        start_node,
                    }) = this.dragging_node
                    {
                        let dx = (ev.position.x - start_mouse.x) / this.viewport.zoom;
                        let dy = (ev.position.y - start_mouse.y) / this.viewport.zoom;
                        if let Some(node) = this.graph.get_node_mut(node_id.clone()) {
                            node.x = start_node.x + dx;
                            node.y = start_node.y + dy;
                            cx.notify();
                        }
                    } else if let Some(Panning {
                        start_mouse,
                        start_offset,
                    }) = this.panning
                    {
                        let dx = ev.position.x - start_mouse.x;
                        let dy = ev.position.y - start_mouse.y;

                        this.viewport.offset.x = start_offset.x + dx;
                        this.viewport.offset.y = start_offset.y + dy;
                        cx.notify();
                    }
                });
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                cx.update_entity(&entry2, |this, cx| {
                    if this.dragging_node.is_some() {
                        this.dragging_node = None;
                        cx.notify();
                    }

                    if this.connecting.is_some() {
                        this.connecting = None;
                        cx.notify();
                    }
                    if this.panning.is_some() {
                        this.panning = None;
                        cx.notify();
                    }
                });
            })
            .on_scroll_wheel(move |ev, _, app| {
                app.update_entity(&entity3, |this, cx| {
                    let cursor = ev.position;

                    let before = this.viewport.screen_to_world(cursor);

                    let delta = f32::from(ev.delta.pixel_delta(px(1.0)).y);
                    if delta == 0.0 {
                        return;
                    }
                    let zoom_delta = if delta > 0.0 { 0.9 } else { 1.1 };

                    this.viewport.zoom *= zoom_delta;

                    this.viewport.zoom = this.viewport.zoom.clamp(0.7, 3.0);

                    let after = this.viewport.world_to_screen(before);

                    this.viewport.offset.x += cursor.x - after.x;
                    this.viewport.offset.y += cursor.y - after.y;

                    cx.notify();
                });
            })
            .child(self.render_grid(window))
            .child(self.render_connecting_edge())
            .child(self.render_edges())
            .children(self.render_nodes(this_cx))
    }
}
