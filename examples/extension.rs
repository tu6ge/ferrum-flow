use ferrum_flow::*;
use gpui::{
    AnyElement, AppContext as _, Application, Element as _, ParentElement as _, Styled,
    WindowOptions, div, rgb, white,
};

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();

        graph
            .create_node("number")
            .position(100.0, 100.0)
            .size(300.0, 150.0)
            .output()
            .build(&mut graph);

        graph
            .create_node("")
            .position(300.0, 400.0)
            .input()
            .build(&mut graph);

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|fc| {
                let mut flow = FlowCanvas::new(graph, fc)
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(BackgroundPlugin::new())
                    .plugin(NodePlugin::new())
                    .plugin(PortInteractionPlugin::new())
                    .plugin(EdgePlugin::new())
                    .plugin(DeletePlugin::new())
                    .plugin(HistoryPlugin::new())
                    .register_node("number", NumberNode {});
                flow.init_plugins();
                flow
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
}
