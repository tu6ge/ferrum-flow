use gpui::{Bounds, Pixels, Point, px};

use crate::canvas::edge::EdgeGeometry;

fn vec_sub(a: Point<Pixels>, b: Point<Pixels>) -> (f32, f32) {
    (f32::from(a.x - b.x), f32::from(a.y - b.y))
}

fn vec_dot(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn vec_length(v: (f32, f32)) -> f32 {
    (v.0 * v.0 + v.1 * v.1).sqrt()
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
