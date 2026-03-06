use gpui::*;

use crate::graph::Graph;

pub struct FlowCanvas {
    pub graph: Graph,
}

impl FlowCanvas {
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }

    fn render_nodes(
        &self,
        _cx: &mut Context<Self>,
    ) -> Vec<impl IntoElement> {
        self.graph
            .nodes
            .values()
            .map(|node| {
                div()
                    .absolute()
                    .left(px(node.x))
                    .top(px(node.y))
                    .w(px(120.0))
                    .h(px(60.0))
                    .bg(gpui::black())
                    .rounded(px(6.0))
                    .child(
                        div()
                            .child("Node")
                            .text_color(gpui::white())
                    )
            })
            .collect()
    }
}

impl Render for FlowCanvas {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(gpui::blue())
            .children(self.render_nodes(cx))
    }
}