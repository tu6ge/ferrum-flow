use ferrum_flow::*;
use gpui::{AppContext as _, Application, WindowOptions};
use serde_json::json;

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph
            .create_node("")
            .position(100.0, 100.0)
            .input()
            .output()
            .data(json!({ "label": "Hello World" }))
            .build(&mut graph);

        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .plugins_core()
                    .build()
            })
        })
        .unwrap();
    });
}
