use flow_rs::*;
use gpui::*;

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();
        graph.add_node(Node::new(1, 100.0, 100.0).node_type("number"));
        graph.add_point(Port::new_output(1, 1, 0));

        graph.add_node(Node::new(2, 300.0, 400.0));
        graph.add_point(Port::new_input(2, 2, 0));

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
