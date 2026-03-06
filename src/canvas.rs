use gpui::*;

use crate::{Edge, EdgeId, Node, NodeId, graph::Graph};

pub struct FlowCanvas {
    pub graph: Graph,
    pub drag_target: Option<(NodeId, Point<Pixels>)>,
    connecting: Option<Connecting>,
}

#[derive(Debug, Clone)]
struct Connecting {
    node_id: NodeId,
    port_id: String,
    mouse: Point<Pixels>,
}

impl FlowCanvas {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            drag_target: None,
            connecting: None,
        }
    }

    fn render_nodes(&self, this_cx: &mut Context<Self>) -> Vec<impl IntoElement> {
        self.graph
            .nodes
            .values()
            .map(|node| {
                let entry = this_cx.entity();
                let entry_id = this_cx.entity_id();
                let node_id = node.id;
                let node_x = node.x;
                let node_y = node.y;
                div()
                    .absolute()
                    .left(node.x)
                    .top(node.y)
                    .on_mouse_down(MouseButton::Left, move |ev, _win, cx| {
                        //println!("mouse down");
                        cx.stop_propagation(); // 防止触发画布的点击事件
                        let offset = ev.position - Point::new(node_x, node_y);

                        cx.update_entity(&entry, |this: &mut Self, _| {
                            this.drag_target = Some((node_id, offset));
                        });
                        cx.notify(entry_id);
                    })
                    .w(px(120.0))
                    .h(px(60.0))
                    .bg(gpui::black())
                    .rounded(px(6.0))
                    .child(self.render_ports(node, this_cx))
                    .child(div().child("Node").text_color(gpui::white()))
            })
            .collect()
    }
    fn render_ports(&self, node: &Node, this_cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .size_full()
            .children(node.outputs.iter().map(|port| {
                let entity = this_cx.entity();
                let entity_id = this_cx.entity_id();
                let node_id = node.id;
                let port_id = port.id.clone();
                div()
                    .absolute()
                    .left(port.point.x - px(6.0))
                    .top(port.point.y - px(6.0))
                    .w(px(12.0))
                    .h(px(12.0))
                    .rounded_full()
                    .bg(rgb(0xdddddd))
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        cx.stop_propagation();
                        cx.update_entity(&entity, |this, _| {
                            this.connecting = Some(Connecting {
                                node_id,
                                port_id: port_id.clone(),
                                mouse: event.position.clone(),
                            });
                        });
                        cx.notify(entity_id);
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
                .map(|(node, port)| Point::new(node.x + port.point.x, node.y + port.point.y))
        } else {
            None
        }
    }
    fn render_connecting_edge(&self) -> impl IntoElement {
        if let Some(connect) = &self.connecting
            && let Some(start) = self.port_position()
        {
            let mouse = connect.mouse;
            canvas(
                |_, _, _| {},
                move |_, _, win, _| {
                    let mut line = PathBuilder::stroke(px(1.0));
                    line.move_to(start);
                    line.curve_to(mouse, Point::new(start.x, start.y + px(50.0)));

                    if let Ok(line) = line.build() {
                        win.paint_path(line, rgb(0xdddddd));
                    }
                },
            )
        } else {
            canvas(|_, _, _| {}, |_, _, _, _| {})
        }
    }
    fn render_edges(&self) -> impl IntoElement {
        let graph = self.graph.clone();

        canvas(
            |_, _, _| graph,
            |_, graph, win, _| {
                for (
                    _,
                    Edge {
                        source_node,
                        target_node,
                        ..
                    },
                ) in graph.edges.iter()
                {
                    let mut line = PathBuilder::stroke(px(1.0));

                    let Some(source_node) = graph.get_node(&source_node) else {
                        return;
                    };
                    let Some(target_node) = graph.get_node(&target_node) else {
                        return;
                    };
                    let source_point = source_node.outputs.first().unwrap().point;
                    line.move_to(Point::new(
                        source_node.x + source_point.x,
                        source_node.y + source_point.y,
                    ));
                    let target_point = target_node.inputs.first().unwrap().point;
                    line.curve_to(
                        Point::new(
                            target_node.x + target_point.x,
                            target_node.y + target_point.y,
                        ),
                        Point::new(
                            source_node.x + source_point.x,
                            source_node.y + source_point.y + px(60.0),
                        ),
                    );

                    if let Ok(line) = line.build() {
                        win.paint_path(line, rgb(0xdddddd));
                    }
                }
            },
        )
    }
}

impl Render for FlowCanvas {
    fn render(&mut self, _window: &mut Window, this_cx: &mut Context<Self>) -> impl IntoElement {
        let entry = this_cx.entity();
        let entry2 = entry.clone();
        let entity_id = this_cx.entity_id();
        div()
            .size_full()
            .bg(gpui::blue())
            .on_mouse_move(move |ev, _, cx| {
                //println!("mouse move");
                cx.update_entity(&entry, |this, _| {
                    if let Some(connect) = &mut this.connecting {
                        connect.mouse = ev.position;
                    } else if let Some((node_id, offset)) = this.drag_target {
                        let new_pos = ev.position - offset;
                        if let Some(node) = this.graph.get_node_mut(node_id.clone()) {
                            node.x = new_pos.x.into();
                            node.y = new_pos.y.into();
                        }
                    }
                });
                cx.notify(entity_id);
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                cx.update_entity(&entry2, |this, cx| {
                    if this.drag_target.is_some() {
                        this.drag_target = None;
                        cx.notify();
                    }

                    if let Some(connecting) = &this.connecting {
                        let edge = Edge {
                            id: EdgeId(1),
                            source_node: connecting.node_id,
                            source_port: connecting.port_id.clone(),
                            target_node: NodeId(2),
                            target_port: "input".into(),
                        };

                        this.graph.add_edge(edge);
                        this.connecting = None;
                        cx.notify();
                    }
                });
            })
            .children(self.render_nodes(this_cx))
            .child(self.render_connecting_edge())
            .child(self.render_edges())
    }
}
