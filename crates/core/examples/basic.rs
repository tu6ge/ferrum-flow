use ferrum_flow::*;
use gpui::{AppContext as _, Application, Size, WindowOptions, px};

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph
            .create_node("")
            .position(100.0, 100.0)
            .output()
            .output()
            .output_with(PortPosition::Bottom, Size::new(px(20.0), px(20.0)))
            .output_at(PortPosition::Bottom)
            .build(&mut graph);

        graph
            .create_node("")
            .position(300.0, 400.0)
            .input()
            .input_at(PortPosition::Top)
            .input_at(PortPosition::Top)
            .output()
            .output_at(PortPosition::Bottom)
            .output_at(PortPosition::Bottom)
            .build(&mut graph);

        graph
            .create_node("")
            .position(500.0, 500.0)
            .input()
            .output()
            .build(&mut graph);

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx)
                    .plugin(MinimapPlugin::new())
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(SnapGuidesPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(ZoomControlsPlugin::new())
                    .plugin(BackgroundPlugin::new())
                    .plugin(NodePlugin::new())
                    .plugin(PortInteractionPlugin::new())
                    .plugin(EdgePlugin::new())
                    .plugin(ClipboardPlugin::new())
                    .plugin(ContextMenuPlugin::new())
                    .plugin(SelectAllViewportPlugin::new())
                    .plugin(AlignPlugin::new())
                    .plugin(FocusSelectionPlugin::new())
                    .plugin(FitAllGraphPlugin::new())
                    .plugin(DeletePlugin::new())
                    .plugin(HistoryPlugin::new())
                    .build()
            })
        })
        .unwrap();
    });
}
