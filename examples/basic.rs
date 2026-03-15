use flow_rs::*;
use gpui::{AppContext as _, Application, WindowOptions};

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph.add_node(Node::new(1, 100.0, 100.0));
        graph.add_point(Port::new_output(1, 1, 0));

        graph.add_node(Node::new(2, 300.0, 400.0));
        graph.add_point(Port::new_input(2, 2, 0));
        graph.add_point(Port::new_output(3, 2, 0));

        graph.add_node(Node::new(3, 500.0, 500.0));
        graph.add_point(Port::new_input(4, 3, 0));
        graph.add_point(Port::new_output(5, 3, 0));

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|fc| {
                let mut flow = FlowCanvas::new(graph, fc)
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(Background::new())
                    .plugin(NodePlugin::new())
                    .plugin(PortInteractionPlugin::new())
                    .plugin(PortPlugin::new())
                    .plugin(EdgePlugin::new())
                    .plugin(DeletePlugin::new());
                flow.init_plugins();
                flow
            })
        })
        .unwrap();
    });
}
