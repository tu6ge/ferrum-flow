use gpui::{Bounds, Element, PathBuilder, Pixels, Point, canvas, px, rgb};

use crate::{
    Edge, EdgeId, Node, Port, PortId, PortKind, RenderContext,
    plugin::{FlowEvent, Plugin, PluginContext},
    plugins::edge::command::ClearEdgeCommand,
};

mod command;

use command::SelectEdgeCommand;

pub struct EdgePlugin;

impl EdgePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for EdgePlugin {
    fn name(&self) -> &'static str {
        "edge"
    }
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::plugin::EventResult {
        if let FlowEvent::Input(crate::plugin::InputEvent::MouseDown(ev)) = event {
            let shift = ev.modifiers.shift;
            if let Some(id) = hit_test_get_edge(ev.position, &ctx) {
                ctx.execute_command(SelectEdgeCommand::new(id, shift, &ctx));
                return crate::plugin::EventResult::Stop;
            } else {
                if !shift {
                    ctx.execute_command(ClearEdgeCommand::new(&ctx));
                }
            }
        }
        crate::plugin::EventResult::Continue
    }
    fn priority(&self) -> i32 {
        120
    }
    fn render_layer(&self) -> crate::plugin::RenderLayer {
        crate::plugin::RenderLayer::Edges
    }
    fn render(&mut self, ctx: &mut crate::RenderContext) -> Option<gpui::AnyElement> {
        let edges: Vec<_> = ctx
            .graph
            .edges
            .iter()
            .map(|(k, v)| (*k, edge_geometry2(v, &ctx)))
            .collect();
        let selected_edges = ctx.graph.selected_edge.clone();

        Some(
            canvas(
                |_, _, _| (edges, selected_edges),
                move |_, (edges, selected_edges), win, _| {
                    for (id, geometry) in edges.iter() {
                        let Some(EdgeGeometry { start, c1, c2, end }) = geometry else {
                            return;
                        };
                        let mut line = PathBuilder::stroke(px(1.0));
                        line.move_to(*start);
                        line.cubic_bezier_to(*end, *c1, *c2);

                        let selected = selected_edges.iter().find(|i| **i == *id).is_some();

                        if let Ok(line) = line.build() {
                            win.paint_path(line, rgb(if selected { 0xFF7800 } else { 0xb1b1b8 }));
                        }
                    }
                },
            )
            .into_any(),
        )
    }
}

pub struct EdgeGeometry {
    pub start: Point<Pixels>,
    pub c1: Point<Pixels>,
    pub c2: Point<Pixels>,
    pub end: Point<Pixels>,
}

fn edge_geometry(edge: &Edge, ctx: &PluginContext) -> Option<EdgeGeometry> {
    let Edge {
        source_port,
        target_port,
        ..
    } = edge;

    let start = port_screen_position(*source_port, &ctx)?;
    let end = port_screen_position(*target_port, &ctx)?;

    Some(EdgeGeometry {
        start,
        c1: start + Point::new(px(50.0), px(0.0)),
        c2: end - Point::new(px(50.0), px(0.0)),
        end,
    })
}

fn edge_geometry2(edge: &Edge, ctx: &RenderContext) -> Option<EdgeGeometry> {
    let Edge {
        source_port,
        target_port,
        ..
    } = edge;

    let start = port_screen_position2(*source_port, &ctx)?;
    let end = port_screen_position2(*target_port, &ctx)?;

    Some(EdgeGeometry {
        start,
        c1: start + Point::new(px(50.0), px(0.0)),
        c2: end - Point::new(px(50.0), px(0.0)),
        end,
    })
}

fn port_screen_position(port_id: PortId, ctx: &PluginContext) -> Option<Point<Pixels>> {
    let port = &ctx.graph.ports.get(&port_id)?;
    let node = &ctx.nodes().get(&port.node_id)?;

    let node_pos = node.point();

    let offset = port_offset(node, port);

    Some(ctx.world_to_screen(node_pos + offset))
}
fn port_screen_position2(port_id: PortId, ctx: &RenderContext) -> Option<Point<Pixels>> {
    let port = &ctx.graph.ports.get(&port_id)?;
    let node = &ctx.nodes().get(&port.node_id)?;

    let node_pos = node.point();

    let offset = port_offset(node, port);

    Some(ctx.world_to_screen(node_pos + offset))
}

pub fn port_offset(node: &Node, port: &Port) -> Point<Pixels> {
    let node_size = node.size;

    match port.kind {
        PortKind::Input => Point::new(px(0.0), node_size.height / 2.0),

        PortKind::Output => Point::new(node_size.width, node_size.height / 2.0),
    }
}

fn hit_test_get_edge(mouse: Point<Pixels>, ctx: &PluginContext) -> Option<EdgeId> {
    for edge in ctx.graph.edges.values() {
        let Some(geom) = edge_geometry(edge, ctx) else {
            continue;
        };

        let bound = edge_bounds(&geom);
        if !bound.contains(&mouse) {
            continue;
        }

        if hit_test_edge(mouse, edge, ctx) {
            return Some(edge.id);
        }
    }

    None
}

pub fn edge_bounds(geom: &EdgeGeometry) -> Bounds<Pixels> {
    let min_x = geom.start.x.min(geom.end.x).min(geom.c1.x).min(geom.c2.x);
    let max_x = geom.start.x.max(geom.end.x).max(geom.c1.x).max(geom.c2.x);

    let min_y = geom.start.y.min(geom.end.y).min(geom.c1.y).min(geom.c2.y);
    let max_y = geom.start.y.max(geom.end.y).max(geom.c1.y).max(geom.c2.y);

    Bounds::from_corners(
        Point::new(min_x - px(10.0), min_y - px(10.0)),
        Point::new(max_x + px(10.0), max_y + px(10.0)),
    )
}

fn hit_test_edge(mouse: Point<Pixels>, edge: &Edge, ctx: &PluginContext) -> bool {
    let Some(geom) = edge_geometry(edge, ctx) else {
        return false;
    };

    let points = sample_bezier(&geom, 20);

    for segment in points.windows(2) {
        let d = distance_to_segment(mouse, segment[0], segment[1]);

        if d < 8.0 {
            return true;
        }
    }

    false
}

fn sample_bezier(geom: &EdgeGeometry, steps: usize) -> Vec<Point<Pixels>> {
    let mut points = Vec::new();

    for i in 0..=steps {
        let t = i as f32 / steps as f32;

        let x = (1.0 - t).powi(3) * geom.start.x
            + 3.0 * (1.0 - t).powi(2) * t * geom.c1.x
            + 3.0 * (1.0 - t) * t * t * geom.c2.x
            + t.powi(3) * geom.end.x;

        let y = (1.0 - t).powi(3) * geom.start.y
            + 3.0 * (1.0 - t).powi(2) * t * geom.c1.y
            + 3.0 * (1.0 - t) * t * t * geom.c2.y
            + t.powi(3) * geom.end.y;

        points.push(Point::new(x, y));
    }

    points
}
pub fn distance_to_segment(p: Point<Pixels>, a: Point<Pixels>, b: Point<Pixels>) -> f32 {
    let ap = vec_sub(p, a);
    let ab = vec_sub(b, a);

    let ab_len2 = ab.0 * ab.0 + ab.1 * ab.1;

    if ab_len2 == 0.0 {
        return vec_length(ap);
    }

    let t = (vec_dot(ap, ab) / ab_len2).clamp(0.0, 1.0);

    let closest = Point::new(f32::from(a.x) + ab.0 * t, f32::from(a.y) + ab.1 * t);

    let dx = f32::from(p.x) - closest.x;
    let dy = f32::from(p.y) - closest.y;

    (dx * dx + dy * dy).sqrt()
}

fn vec_sub(a: Point<Pixels>, b: Point<Pixels>) -> (f32, f32) {
    (f32::from(a.x - b.x), f32::from(a.y - b.y))
}

fn vec_dot(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn vec_length(v: (f32, f32)) -> f32 {
    (v.0 * v.0 + v.1 * v.1).sqrt()
}
