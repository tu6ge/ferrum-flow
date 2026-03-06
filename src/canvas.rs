use gpui::*;

use crate::{NodeId, graph::Graph};

pub struct FlowCanvas {
    pub graph: Graph,
    pub drag_target: Option<(NodeId, Point<Pixels>)>,
}

impl FlowCanvas {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            drag_target: None,
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
                    .child(div().child("Node").text_color(gpui::white()))
            })
            .collect()
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
                    if let Some((node_id, offset)) = this.drag_target {
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
                cx.update_entity(&entry2, |this, _| {
                    this.drag_target = None;
                });
            })
            .children(self.render_nodes(this_cx))
    }
}
