use gpui::{Bounds, Path, PathBuilder, Pixels, Point, Size, px};

use crate::{Node, Port, PortId, PortKind, RenderContext};

pub fn port_offset(node: &Node, port: &Port) -> Point<Pixels> {
    let node_size = node.size;

    match port.kind {
        PortKind::Input => Point::new(px(0.0), node_size.height / 2.0),

        PortKind::Output => Point::new(node_size.width, node_size.height / 2.0),
    }
}

pub fn port_screen_position(port_id: PortId, ctx: &RenderContext) -> Option<Point<Pixels>> {
    let port = &ctx.graph.ports[&port_id];
    let Some(node) = &ctx.graph.nodes().get(&port.node_id) else {
        return None;
    };

    let node_pos = node.point();

    let offset = port_offset(node, port);

    Some(ctx.viewport.world_to_screen(node_pos + offset))
}

pub fn port_screen_bounds(
    port_id: PortId,
    ctx: &crate::plugin::PluginContext,
) -> Option<Bounds<Pixels>> {
    let port = &ctx.graph.ports[&port_id];
    let Some(node) = &ctx.graph.nodes().get(&port.node_id) else {
        return None;
    };

    let node_pos = node.point();

    let offset = port_offset(node, port);

    Some(Bounds::new(
        node_pos + offset - Point::new(px(6.0), px(6.0)),
        Size::new(px(12.0), px(12.0)),
    ))
}

pub fn edge_bezier(
    start: Point<Pixels>,
    end: Point<Pixels>,
) -> Result<Path<Pixels>, anyhow::Error> {
    let mut line = PathBuilder::stroke(px(1.0));
    line.move_to(start);
    line.cubic_bezier_to(
        end,
        Point::new(start.x + px(50.0), start.y),
        Point::new(end.x - px(50.0), end.y),
    );

    line.build()
}
