use ferrum_flow::*;
use ferrum_flow_sync_plugin::{Assets, YrsSyncPlugin};
use gpui::{AppContext as _, Application, Size, WindowOptions, px};

fn main() {
    Application::new().with_assets(Assets).run(|cx| {
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

        graph = if std::env::var("IS_INIT").unwrap_or_default() == "1" {
            graph
        } else {
            Graph::new()
        };

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(Graph::new(), ctx)
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(SnapGuidesPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(ZoomControlsPlugin::new())
                    .plugin(BackgroundPlugin::new())
                    .plugin(NodePlugin::new())
                    .plugin(MinimapPlugin::new())
                    .plugin(ClipboardPlugin::new())
                    .plugin(ContextMenuPlugin::new())
                    .plugin(SelectAllViewportPlugin::new())
                    .plugin(AlignPlugin::new())
                    .plugin(FocusSelectionPlugin::new())
                    .plugin(FitAllGraphPlugin::new())
                    .plugin(PortInteractionPlugin::new())
                    .plugin(EdgePlugin::new())
                    .plugin(DeletePlugin::new())
                    .plugin(HistoryPlugin::new())
                    .sync_plugin(YrsSyncPlugin::new(graph, "ws://127.0.0.1:9001"))
                    .build()
            })
        })
        .unwrap();
    });
}
