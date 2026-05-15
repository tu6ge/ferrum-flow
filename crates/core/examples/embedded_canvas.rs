//! Embedded [`FlowCanvas`] inside a parent shell with **simple bidirectional wiring**:
//!
//! - **Canvas → parent** — [`FlowCanvas::set_outbound`] observes [`NodeDragEvent::End`]; a short
//!   status line is copied into the shell when the canvas notifies (via [`gpui::Context::observe`]).
//! - **Parent → canvas** — toolbar click runs [`FlowCanvas::dispatch_command`] ([`CreateNode`] +
//!   [`CreatePort`]) on the canvas entity.
//!
//! Run: `cargo run -p ferrum-flow --example embedded_canvas`

use std::sync::{Arc, Mutex};

use ferrum_flow::*;
use gpui::{
    AppContext as _, Application, Context, Entity, InteractiveElement as _, MouseButton,
    MouseDownEvent, ParentElement as _, Render, Styled as _, Window, WindowOptions, div, px, rgb,
    rgba,
};
use serde_json::json;

/// Top bar is the parent UI; the canvas sits in the inset region below.
struct ParentShell {
    canvas: Entity<FlowCanvas>,
    /// Last message pushed from the canvas outbound hook (see [`ParentShell::new`]).
    last_from_canvas: String,
}

impl ParentShell {
    fn new(canvas: Entity<FlowCanvas>, cx: &mut Context<Self>) -> Self {
        let pending = Arc::new(Mutex::new(Option::<String>::None));
        let pending_for_observe = pending.clone();

        canvas.update(cx, |c, _| {
            let pending_for_outbound = pending.clone();
            c.set_outbound(Some(Box::new(move |ev: &FlowEvent| {
                if ev
                    .as_custom::<NodeDragEvent>()
                    .is_some_and(|e| matches!(e, NodeDragEvent::End))
                {
                    if let Ok(mut slot) = pending_for_outbound.lock() {
                        *slot = Some("Canvas → parent: primary node drag ended".into());
                    }
                }
            })));
        });

        let canvas_watch = canvas.clone();
        cx.observe(&canvas_watch, move |this, _, cx| {
            let msg = pending_for_observe.lock().ok().and_then(|mut m| m.take());
            if let Some(m) = msg {
                this.last_from_canvas = m;
                cx.notify();
            }
        })
        .detach();

        Self {
            canvas,
            last_from_canvas: "Drag a node, then release — canvas reports here.".into(),
        }
    }

    fn add_node_from_parent(
        &mut self,
        _: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (node, ports) = build_node_with_ports_for_dispatch();
        self.canvas.update(cx, |canvas, cx| {
            canvas.dispatch_command(CreateNode::new(node), cx);
            for port in ports {
                canvas.dispatch_command(CreatePort::new(port), cx);
            }
        });
    }
}

/// Build a detached node + ports (same pattern as tests using [`Graph::create_node`] … [`NodeBuilderInGraph::build_raw`]).
fn build_node_with_ports_for_dispatch() -> (Node, Vec<Port>) {
    let mut g = Graph::new();
    let (node, ports, _) = g
        .create_node("default")
        .position(260.0, 140.0)
        .output()
        .data(json!({ "label": "From parent" }))
        .build_raw();
    (node, ports)
}

impl Render for ParentShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        const INSET_LEFT: f32 = 88.0;
        const INSET_TOP: f32 = 104.0;
        const INSET_RIGHT: f32 = 40.0;
        const INSET_BOTTOM: f32 = 36.0;

        let entity = cx.entity().clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x1a1d24))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(12.0))
                    .h(px(52.0))
                    .px(px(16.0))
                    .border_b(px(1.0))
                    .border_color(rgba(0xffffff12))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgba(0xe8ecf1))
                            .child(self.last_from_canvas.clone()),
                    )
                    .child(
                        div()
                            .id("embedded-add-node")
                            .cursor_pointer()
                            .px(px(10.0))
                            .py(px(5.0))
                            .rounded(px(4.0))
                            .bg(rgb(0x2d3548))
                            .text_sm()
                            .text_color(rgba(0xe8ecf1))
                            .child("Add node (parent → canvas)")
                            .on_mouse_down(
                                MouseButton::Left,
                                window.listener_for(&entity, ParentShell::add_node_from_parent),
                            ),
                    ),
            )
            .child(
                div().flex_1().relative().min_h(px(0.0)).child(
                    div()
                        .absolute()
                        .inset(px(12.0))
                        .bg(rgb(0x12151c))
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(rgba(0xffffff10))
                        .child(
                            div()
                                .absolute()
                                .top(px(INSET_TOP))
                                .left(px(INSET_LEFT))
                                .right(px(INSET_RIGHT))
                                .bottom(px(INSET_BOTTOM))
                                .rounded(px(6.0))
                                .border_2()
                                .border_color(rgb(0xc45c26))
                                .child(self.canvas.clone()),
                        ),
                ),
            )
    }
}

fn main() {
    Application::new().run(|cx| {
        let graph = Graph::build(|g| {
            g.create_node("default")
                .position(120.0, 100.0)
                .output()
                .data(json!({ "label": "First Node" }))
                .build();

            g.create_node("default")
                .position(380.0, 260.0)
                .input()
                .data(json!({ "label": "Second Node" }))
                .build();
        });

        cx.open_window(WindowOptions::default(), |window, cx| {
            let canvas = cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .default_plugins()
                    .plugin(MinimapPlugin::new())
                    .plugin(ZoomControlsPlugin::new())
                    .plugin(ClipboardPlugin::new())
                    .plugin(ContextMenuPlugin::new())
                    .plugin(SelectAllViewportPlugin::new())
                    .plugin(AlignPlugin::new())
                    .plugin(FocusSelectionPlugin::new())
                    .plugin(FitAllGraphPlugin::new())
                    .plugin(SnapGuidesPlugin::new())
                    .plugin(ToastPlugin::new())
                    .build()
            });
            cx.new(|ctx| ParentShell::new(canvas, ctx))
        })
        .unwrap();
    });
}
