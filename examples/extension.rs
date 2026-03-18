use ferrum_flow::*;
use gpui::{
    AnyElement, AppContext as _, Application, Element as _, ParentElement as _, Size, Styled,
    WindowOptions, div, px, rgb, white,
};

fn main() {
    Application::new().run(|cx| {
        let mut graph = Graph::new();
        graph.add_node(
            Node::new(1, 100.0, 100.0)
                .set_size(Size::new(px(300.0), px(150.0)))
                .node_type("number"),
        );
        graph.add_point(Port::new_output(1, 1, 0));

        graph.add_node(Node::new(2, 300.0, 400.0));
        graph.add_point(Port::new_input(2, 2, 0));

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|fc| {
                let mut flow = FlowCanvas::new(graph, fc)
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(Background::new())
                    .plugin(NodePlugin::new().register_node("number", NumberNode {}));
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
