use gpui::*;
use flow_rs::*;

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph.add_node(Node {
            id: NodeId(1),
            x: 100.0,
            y: 100.0,
        });

        graph.add_node(Node {
            id: NodeId(2),
            x: 300.0,
            y: 200.0,
        });

        cx.open_window(
            WindowOptions::default(),
            |_,cx| cx.new(|_| FlowCanvas::new(graph)),
        ).unwrap();
    });
}