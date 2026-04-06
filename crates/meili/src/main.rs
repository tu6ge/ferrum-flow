//! Meili — AI agent workflow canvas (FerrumFlow + [`FlowTheme`](ferrum_flow::FlowTheme) via
//! [`plugins::MeiliThemePlugin`] and [`theme::apply_flow_chrome`](crate::theme::apply_flow_chrome)).
//!
//! After a dangling link, node type is chosen with [`plugins::MeiliPortInteractionPlugin`] (Meili fork: blue-dot click),
//! [`plugins::NodeTypePickerPlugin`], and the **gpui-component** [`Select`](gpui_component::select::Select) in [`shell::MeiliShell`].
//! Core `PortInteractionPlugin` is not registered so this works without changing `ferrum-flow`.

mod add_node_dialog;
mod demo_graph;
mod pick_state;
mod plugins;
mod renderers;
mod shell;
mod theme;

use ferrum_flow::*;
use gpui::{AppContext as _, Application, WindowOptions};
use renderers::{WorkflowKind, WorkflowNodeRenderer};

fn main() {
    Application::new().run(|cx| {
        gpui_component::init(cx);

        let mut graph = Graph::new();
        demo_graph::build_sample_workflow(&mut graph);

        cx.open_window(WindowOptions::default(), |window, cx| {
            let canvas = cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .plugin(plugins::MeiliThemePlugin::new())
                    .plugin(plugins::AgentBackgroundPlugin::new())
                    .plugin(plugins::AgentHudPlugin::new())
                    .plugin(MinimapPlugin::new())
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(SnapGuidesPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(ZoomControlsPlugin::new())
                    .plugin(NodePlugin::new())
                    .plugin(plugins::NodeTypePickerPlugin::new())
                    .plugin(plugins::MeiliPortInteractionPlugin::new())
                    .plugin(EdgePlugin::new())
                    .plugin(ClipboardPlugin::new())
                    .plugin(
                        ContextMenuPlugin::new().canvas_row("Add node…", |ctx, world| {
                            crate::add_node_dialog::open_at(world);
                            ctx.notify();
                        }),
                    )
                    .plugin(plugins::MeiliAddNodePlugin::new())
                    .plugin(SelectAllViewportPlugin::new())
                    .plugin(AlignPlugin::new())
                    .plugin(FocusSelectionPlugin::new())
                    .plugin(FitAllGraphPlugin::new())
                    .plugin(DeletePlugin::new())
                    .plugin(HistoryPlugin::new())
                    .node_renderer("io_start", WorkflowNodeRenderer::new(WorkflowKind::IoStart))
                    .node_renderer("io_end", WorkflowNodeRenderer::new(WorkflowKind::IoEnd))
                    .node_renderer("agent", WorkflowNodeRenderer::new(WorkflowKind::Agent))
                    .node_renderer("llm", WorkflowNodeRenderer::new(WorkflowKind::Llm))
                    .node_renderer("tool", WorkflowNodeRenderer::new(WorkflowKind::Tool))
                    .node_renderer("router", WorkflowNodeRenderer::new(WorkflowKind::Router))
                    .node_renderer("", WorkflowNodeRenderer::new(WorkflowKind::Step))
                    .build()
            });
            cx.new(|ctx| shell::window_root(canvas, window, ctx))
        })
        .unwrap();
    });
}
