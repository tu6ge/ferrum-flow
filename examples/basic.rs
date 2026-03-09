use flow_rs::*;
use gpui::*;

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph.add_node(
            Node::new(1, 100.0, 100.0).output("1".into(), Point::new(px(60.0), px(60.0))),
        );

        graph.add_node(
            Node::new(2, 300.0, 400.0)
                .input("1".into(), Point::new(px(60.0), px(0.0)))
                .output("2".into(), Point::new(px(60.0), px(60.0))),
        );

        graph.add_node(
            Node::new(3, 500.0, 500.0)
                .input("1".into(), Point::new(px(60.0), px(0.0)))
                .output("2".into(), Point::new(px(60.0), px(60.0))),
        );

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|fc| FlowCanvas::new(graph, fc))
        })
        .unwrap();
    });
}
