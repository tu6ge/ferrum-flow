use ferrum_flow::*;
use gpui::{AppContext as _, Application, WindowOptions};

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        for j in 0..100 {
            for i in 0..100 {
                graph
                    .create_node("")
                    .position(200.0 * i as f32, 200.0 * j as f32)
                    .input()
                    .output()
                    .build(&mut graph);
            }
        }

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
