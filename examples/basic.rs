use ferrum_flow::*;
use gpui::{AppContext as _, Application, WindowOptions};

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph
            .create_node("")
            .position(100.0, 100.0)
            .output()
            .output()
            .output_at(PortPosition::Bottom)
            .output_at(PortPosition::Bottom)
            .build(&mut graph);

        graph
            .create_node("")
            .position(300.0, 400.0)
            .input()
            .output()
            .build(&mut graph);

        graph
            .create_node("")
            .position(500.0, 500.0)
            .input()
            .output()
            .build(&mut graph);

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|fc| {
                let mut flow = FlowCanvas::new(graph, fc)
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(Background::new())
                    .plugin(NodePlugin::new())
                    .plugin(PortInteractionPlugin::new())
                    .plugin(EdgePlugin::new())
                    .plugin(DeletePlugin::new())
                    .plugin(HistoryPlugin::new());
                flow.init_plugins();
                flow
            })
        })
        .unwrap();
    });
}
