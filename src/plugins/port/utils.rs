use gpui::{Bounds, Path, PathBuilder, Pixels, Point, Size, px};

use crate::{PortId, PortPosition, RenderContext, Viewport, plugins::edge::get_control_point};

pub fn port_screen_position(port_id: PortId, ctx: &RenderContext) -> Option<Point<Pixels>> {
    let port = &ctx.graph.ports.get(&port_id)?;
    let node = &ctx.nodes().get(&port.node_id)?;

    let node_pos = node.point();

    let offset = ctx.port_offset_cached(&port.node_id, &port_id)?;

    Some(ctx.viewport.world_to_screen(node_pos + offset))
}

pub fn port_screen_bounds(
    port_id: PortId,
    ctx: &crate::plugin::PluginContext,
) -> Option<Bounds<Pixels>> {
    let port = &ctx.graph.ports.get(&port_id)?;
    let node = &ctx.nodes().get(&port.node_id)?;

    let node_pos = node.point();

    let offset = ctx.port_offset_cached(&port.node_id, &port_id)?;

    Some(Bounds::new(
        node_pos + offset - Point::new(px(6.0), px(6.0)),
        Size::new(px(12.0), px(12.0)),
    ))
}

pub fn edge_bezier(
    start: Point<Pixels>,
    start_position: PortPosition,
    end: Point<Pixels>,
    viewport: &Viewport,
) -> Result<Path<Pixels>, anyhow::Error> {
    let control_a = get_control_point(start, start_position, viewport);
    let mut line = PathBuilder::stroke(px(1.0));
    line.move_to(start);
    line.cubic_bezier_to(end, control_a, Point::new(end.x - px(50.0), end.y));

    line.build()
}
