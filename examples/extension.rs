use flow_rs::*;
use gpui::*;

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();
        graph.add_node(
            Node::new(1, 100.0, 100.0)
                .output("1".into(), Point::new(px(60.0), px(60.0)))
                .node_type("number"),
        );

        graph.add_node(Node::new(2, 300.0, 400.0).input("1".into(), Point::new(px(60.0), px(0.0))));

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|fc| FlowCanvas::new(graph, fc).register_node("number", NumberNode {}))
        })
        .unwrap();
    });
}

pub struct NumberNode;

impl NodeRenderer for NumberNode {
    fn size(&self, _node: &Node) -> Size<Pixels> {
        Size::new(px(160.0), px(80.0))
    }

    fn render(&self, _node: &Node, cx: &mut NodeRenderContext) -> AnyElement {
        div()
            .size_full()
            .bg(rgb(0x505078))
            .rounded(cx.rounded)
            .child(div().child("Number Node").text_color(white()))
            .into_any()
    }
}
