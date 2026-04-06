//! Fit the viewport to all nodes after load (same geometry as core FitAll, local to this crate).

use ferrum_flow::PluginContext;
use gpui::{Point, px};

const ZOOM_MIN: f32 = 0.7;
const ZOOM_MAX: f32 = 3.0;
const MARGIN_RATIO: f32 = 0.08;

fn graph_bounds(ctx: &PluginContext) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    let mut any = false;

    for n in ctx.graph.nodes().values() {
        let x: f32 = n.x.into();
        let y: f32 = n.y.into();
        let w: f32 = n.size.width.into();
        let h: f32 = n.size.height.into();
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);
        any = true;
    }

    if !any {
        return None;
    }

    Some((
        min_x,
        min_y,
        (max_x - min_x).max(1.0),
        (max_y - min_y).max(1.0),
    ))
}

fn frame_params(
    win_w: f32,
    win_h: f32,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
) -> Option<(f32, Point<gpui::Pixels>)> {
    if win_w <= 0.0 || win_h <= 0.0 {
        return None;
    }

    let bw_safe = bw.max(1.0);
    let bh_safe = bh.max(1.0);

    let avail_w = win_w * (1.0 - 2.0 * MARGIN_RATIO);
    let avail_h = win_h * (1.0 - 2.0 * MARGIN_RATIO);
    let z = (avail_w / bw_safe)
        .min(avail_h / bh_safe)
        .clamp(ZOOM_MIN, ZOOM_MAX);

    let cx = bx + bw / 2.0;
    let cy = by + bh / 2.0;
    let center_x = win_w / 2.0;
    let center_y = win_h / 2.0;
    let new_offset = Point::new(px(center_x - cx * z), px(center_y - cy * z));

    Some((z, new_offset))
}

/// Update viewport directly (not on undo stack); one-shot after open or sample swap.
pub fn fit_entire_graph_in_viewport(ctx: &mut PluginContext) {
    let Some(wb) = ctx.viewport.window_bounds else {
        return;
    };
    let win_w: f32 = wb.size.width.into();
    let win_h: f32 = wb.size.height.into();
    let Some((bx, by, bw, bh)) = graph_bounds(ctx) else {
        return;
    };
    let Some((z, new_offset)) = frame_params(win_w, win_h, bx, by, bw, bh) else {
        return;
    };

    ctx.viewport.zoom = z;
    ctx.viewport.offset = new_offset;
}
