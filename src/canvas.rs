use gpui::*;

use crate::{Edge, EdgeId, Node, NodeId, graph::Graph, viewport::Viewport};

pub struct FlowCanvas {
    pub graph: Graph,
    dragging_node: Option<DraggingNode>,
    connecting: Option<Connecting>,

    viewport: Viewport,
    panning: Option<Panning>,
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
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            dragging_node: None,
            connecting: None,
            viewport: Viewport::new(),
            panning: None,
        }
    }

    fn render_nodes(&self, this_cx: &mut Context<Self>) -> Vec<impl IntoElement> {
        self.graph
            .nodes
            .values()
            .map(|node| {
                let entry = this_cx.entity();
                let node_id = node.id;
                let node_point = node.point();
                let screen = self.viewport.world_to_screen(node.point());
                let node_x = screen.x;
                let node_y = screen.y;
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
                            cx.notify();
                        });
                    })
                    .w(px(120.0 * self.viewport.zoom))
                    .h(px(60.0 * self.viewport.zoom))
                    .bg(white())
                    .rounded(px(6.0))
                    .border(px(1.5))
                    .border_color(rgb(0x1A192B))
                    .child(self.render_ports(node, this_cx))
                    .child(
                        div()
                            .child(format!("Node {}", node_id))
                            .text_color(rgb(0x1A192B)),
                    )
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
                                let edge = Edge {
                                    id: EdgeId(1),
                                    source_node: connecting.node_id,
                                    source_port: connecting.port_id.clone(),
                                    target_node: node_id,
                                    target_port: port_id.clone(),
                                };

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
        let graph = self.graph.clone();
        let viewport = self.viewport.clone();

        canvas(
            |_, _, _| (graph, viewport),
            |_, (graph, viewport), win, _| {
                for (
                    _,
                    Edge {
                        source_node,
                        target_node,
                        source_port,
                        target_port,
                        ..
                    },
                ) in graph.edges.iter()
                {
                    let Some(source_node) = graph.get_node(&source_node) else {
                        return;
                    };
                    let Some(target_node) = graph.get_node(&target_node) else {
                        return;
                    };
                    let source_point = source_node
                        .outputs
                        .iter()
                        .find(|p| p.id == source_port.clone())
                        .map(|p| p.point);
                    let Some(source_point) = source_point else {
                        return;
                    };
                    let target_point = target_node
                        .inputs
                        .iter()
                        .find(|p| p.id == target_port.clone())
                        .map(|p| p.point);
                    let Some(target_point) = target_point else {
                        return;
                    };

                    if let Ok(line) = edge_bezier(
                        viewport.world_to_screen(Point::new(
                            source_node.x + source_point.x,
                            source_node.y + source_point.y,
                        )),
                        viewport.world_to_screen(Point::new(
                            target_node.x + target_point.x,
                            target_node.y + target_point.y,
                        )),
                    ) {
                        win.paint_path(line, rgb(0xb1b1b8));
                    }
                }
            },
        )
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

impl Render for FlowCanvas {
    fn render(&mut self, _window: &mut Window, this_cx: &mut Context<Self>) -> impl IntoElement {
        let entry = this_cx.entity();
        let entry2 = entry.clone();
        let entity3 = entry.clone();
        let entity_mouse_down = entry.clone();
        div()
            .size_full()
            // bg point 9F9FA7
            .bg(gpui::rgb(0xf8f9fb))
            .on_mouse_down(MouseButton::Left, move |ev, _, app| {
                app.update_entity(&entity_mouse_down, |this, cx| {
                    this.panning = Some(Panning {
                        start_mouse: ev.position,
                        start_offset: this.viewport.offset,
                    });
                    cx.notify();
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
            .children(self.render_nodes(this_cx))
            .child(self.render_connecting_edge())
            .child(self.render_edges())
    }
}
