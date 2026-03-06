use flow_rs::*;
use gpui::*;

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph.add_node(Node {
            id: NodeId(1),
            x: 100.0.into(),
            y: 100.0.into(),
        });

        graph.add_node(Node {
            id: NodeId(2),
            x: 300.0.into(),
            y: 200.0.into(),
        });

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| FlowCanvas::new(graph))
        })
        .unwrap();
    });
}
