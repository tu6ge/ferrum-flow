use ferrum_flow::*;
use gpui::{
    AnyElement, AppContext as _, Application, Element as _, ParentElement as _, Styled,
    WindowOptions, div, rgb, white,
};
use serde_json::json;

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph
            .create_node("number")
            .position(100.0, 100.0)
            .size(300.0, 150.0)
            .output()
            .data(json!({ "label": "Number Node" }))
            .build(&mut graph);

        graph
            .create_node("")
            .position(300.0, 400.0)
            .input()
            .build(&mut graph);

        graph
            .create_node("undefined")
            .position(500.0, 500.0)
            .input()
            .output()
            .build(&mut graph);

        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .plugins_core()
                    .plugin(ZoomControlsPlugin::new())
                    .plugin(FocusSelectionPlugin::new())
                    .plugin(FitAllGraphPlugin::new())
                    .plugin(ClipboardPlugin::new())
                    .plugin(ContextMenuPlugin::new())
                    .node_renderer("number", NumberNode {})
                    .build()
            })
        })
        .unwrap();
    });
}

pub struct NumberNode;

impl NodeRenderer for NumberNode {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement {
        let screen = ctx.viewport.world_to_screen(node.point());
        let node_x = screen.x;
        let node_y = screen.y;

        div()
            .absolute()
            .left(node_x)
            .top(node_y)
            .w(node.size.width * ctx.viewport.zoom)
            .h(node.size.height * ctx.viewport.zoom)
            .bg(rgb(0x505078))
            .child(div().child("Number Node").text_color(white()))
            .into_any()
    }

    fn port_render(&self, node: &Node, port: &Port, ctx: &mut RenderContext) -> Option<AnyElement> {
        let size = port.size;
        let position = port_screen_position(node, port.id, &ctx)?;

        Some(
            div()
                .absolute()
                .left(position.x - size.width / 2.0 * ctx.viewport.zoom)
                .top(position.y - size.height / 2.0 * ctx.viewport.zoom)
                .w(size.width * ctx.viewport.zoom)
                .h(size.height * ctx.viewport.zoom)
                .rounded_full()
                .border_1()
                .border_color(rgb(0x1A192B))
                .bg(white())
                .into_any(),
        )
    }
}
