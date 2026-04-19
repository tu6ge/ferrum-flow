use ferrum_flow::*;
use ferrum_flow_sync_plugin::{Assets, YrsSyncPlugin};
use gpui::{
    AnyElement, AppContext as _, Application, Element as _, ParentElement as _, Size, Styled as _,
    WindowOptions, div, px, rgb,
};

/// Renders like the built-in default card, plus a second line with the full node UUID for sync demos.
struct SyncBasicNodeRenderer;

impl NodeRenderer for SyncBasicNodeRenderer {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement {
        let node_id = node.id();
        let selected = ctx.graph.selected_node().contains(&node_id);

        ctx.node_card_shell(node, selected, NodeCardVariant::Default)
            .rounded(px(6.0))
            .border(px(1.5))
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .text_center()
                    .px_2()
                    .gap(px(2.0))
                    .min_h(px(0.0))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x1A192B))
                            .child(default_node_caption(node)),
                    )
                    .child(
                        div()
                            .w_full()
                            .min_w(px(0.0))
                            .text_xs()
                            .text_color(rgb(0x7a7a88))
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(node.id().to_string()),
                    ),
            )
            .into_any()
    }
}

fn main() {
    Application::new().with_assets(Assets).run(|cx| {
        let mut graph = Graph::new();

        graph
            .create_node("sync")
            .position(100.0, 100.0)
            .output()
            .output()
            .output_with(PortPosition::Bottom, Size::new(px(20.0), px(20.0)))
            .output_at(PortPosition::Bottom)
            .build();

        graph
            .create_node("sync")
            .position(300.0, 400.0)
            .input()
            .input_at(PortPosition::Top)
            .input_at(PortPosition::Top)
            .output()
            .output_at(PortPosition::Bottom)
            .output_at(PortPosition::Bottom)
            .build();

        graph
            .create_node("sync")
            .position(500.0, 500.0)
            .input()
            .output()
            .build();

        graph = if std::env::var("IS_INIT").unwrap_or_default() == "1" {
            graph
        } else {
            Graph::new()
        };

        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(Graph::new(), ctx, window)
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
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
                    .node_renderer("sync", SyncBasicNodeRenderer)
                    .sync_plugin(YrsSyncPlugin::new(graph, "ws://127.0.0.1:9001"))
                    .build()
            })
        })
        .unwrap();
    });
}
