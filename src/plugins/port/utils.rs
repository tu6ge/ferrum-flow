use gpui::{Bounds, Path, PathBuilder, Pixels, Point, Size, px};

use crate::{PortId, RenderContext};

pub fn port_screen_position(port_id: PortId, ctx: &RenderContext) -> Option<Point<Pixels>> {
    let port = &ctx.graph.ports.get(&port_id)?;
    let node = &ctx.nodes().get(&port.node_id)?;

    let renderer = ctx.get_node_render(&port.node_id)?;

    let node_pos = node.point();

    let offset = renderer.port_offset(node, port, ctx.graph);

    Some(ctx.viewport.world_to_screen(node_pos + offset))
}

pub fn port_screen_bounds(
    port_id: PortId,
    ctx: &crate::plugin::PluginContext,
) -> Option<Bounds<Pixels>> {
    let port = &ctx.graph.ports.get(&port_id)?;
    let node = &ctx.nodes().get(&port.node_id)?;

    let renderer = ctx.get_node_render(&port.node_id)?;

    let node_pos = node.point();

    let offset = renderer.port_offset(node, port, ctx.graph);

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
