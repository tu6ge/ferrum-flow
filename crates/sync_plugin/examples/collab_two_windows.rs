//! One process: sample WebSocket relay + **two** GPUI windows on the same doc (`ws://127.0.0.1:9001`).
//!
//! Run:
//! ```bash
//! cargo run -p ferrum-flow-sync-plugin --features dev-ws-relay --example collab_two_windows
//! ```
//!
//! - By default the **left** window starts with the same seeded graph as `IS_INIT=1` in `basic`; the **right**
//!   window starts empty and catches up from the relay. Set `IS_INIT=0` so **both** start from an empty graph
//!   (same as `basic` without `IS_INIT`).

use std::thread;
use std::time::Duration;

use ferrum_flow::*;
use ferrum_flow_sync_plugin::{PresenceConfig, YrsSyncPlugin, run_dev_ws_relay};
use gpui::{
    AnyElement, AppContext as _, Application, Bounds, Element as _, ParentElement as _, Pixels,
    Point, Size, Styled as _, TitlebarOptions, WindowBounds, WindowOptions, div, px, rgb,
};

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

fn demo_seed_graph() -> Graph {
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

    graph
}

fn seed_left_window() -> bool {
    match std::env::var("IS_INIT") {
        Ok(s) if s == "0" => false,
        _ => true,
    }
}

fn window_options(title: &'static str, origin: Point<Pixels>, focus: bool) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin,
            size: Size {
                width: px(720.0),
                height: px(640.0),
            },
        })),
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            ..Default::default()
        }),
        focus,
        ..Default::default()
    }
}

fn main() {
    thread::Builder::new()
        .name("dev-ws-relay".into())
        .spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            rt.block_on(run_dev_ws_relay());
        })
        .expect("spawn relay thread");

    thread::sleep(Duration::from_millis(200));

    let left_initial = if seed_left_window() {
        demo_seed_graph()
    } else {
        Graph::new()
    };
    let right_initial = Graph::new();

    Application::new().run(|cx| {
        let build_canvas = |sync_seed: Graph, user_name: &'static str, user_color: u32| {
            move |window: &mut gpui::Window, cx: &mut gpui::App| {
                cx.new(|ctx| {
                    let presence = PresenceConfig::new()
                        .with_local_name(user_name)
                        .with_local_color(user_color)
                        .with_show_remote_name(true);
                    FlowCanvas::builder(Graph::new(), ctx, window)
                        .plugin(SelectionPlugin::new())
                        .plugin(NodeInteractionPlugin::new())
                        .plugin(ViewportPlugin::new())
                        .plugin(ZoomControlsPlugin::new())
                        .plugin(BackgroundPlugin::new())
                        .plugin(NodePlugin::new())
                        //.plugin(MinimapPlugin::new())
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
                        .sync_plugin(
                            YrsSyncPlugin::new(sync_seed, "ws://127.0.0.1:9001")
                                .with_presence_config(presence),
                        )
                        .build()
                })
            }
        };

        cx.open_window(
            window_options(
                "Ferrum sync — client A",
                Point::new(px(40.0), px(40.0)),
                true,
            ),
            build_canvas(left_initial, "client-a", 0xFF6B6B),
        )
        .unwrap();

        cx.open_window(
            window_options(
                "Ferrum sync — client B",
                Point::new(px(800.0), px(40.0)),
                false,
            ),
            build_canvas(right_initial, "client-b", 0x4ECDC4),
        )
        .unwrap();
    });
}
