use ferrum_flow::*;
use gpui::{
    AppContext as _, Application, Context, Entity, ParentElement as _, Render, Styled as _, Window,
    WindowOptions, div, px, rgb, rgba,
};
use serde_json::json;

/// The top bar is the parent UI; the canvas area in the bottom region uses absolute positioning to leave **left/top/right/bottom** margins (to change the canvas's starting point in the window).
struct ParentShell {
    canvas: Entity<FlowCanvas>,
}

impl ParentShell {
    fn new(canvas: Entity<FlowCanvas>) -> Self {
        Self { canvas }
    }
}

impl Render for ParentShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        const INSET_LEFT: f32 = 88.0;
        const INSET_TOP: f32 = 104.0;
        const INSET_RIGHT: f32 = 40.0;
        const INSET_BOTTOM: f32 = 36.0;

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
                    .h(px(52.0))
                    .px(px(16.0))
                    .border_b(px(1.0))
                    .border_color(rgba(0xffffff12))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgba(0xe8ecf1))
                            .child(format!(
                                "Parent Shell Toolbar — Canvas inset: left={INSET_LEFT}, top={INSET_TOP}, right={INSET_RIGHT}, bottom={INSET_BOTTOM}"
                            )),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .relative()
                    .min_h(px(0.0))
                    .child(
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
            cx.new(|_ctx| ParentShell::new(canvas))
        })
        .unwrap();
    });
}
