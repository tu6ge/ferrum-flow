use std::collections::HashSet;

use gpui::{Element as _, PathBuilder, Point, canvas, px, rgb};

use crate::{
    Graph, Node, NodeId, Viewport,
    alignment_guides::AlignmentGuides,
    plugin::{Plugin, RenderContext, RenderLayer},
};

/// Screen-space snap distance; world threshold = this / zoom.
const SNAP_SCREEN_PX: f32 = 5.0;
const GUIDE_COLOR: u32 = 0xff4081;

fn visible_world_bounds(viewport: &Viewport) -> Option<(f32, f32, f32, f32)> {
    let wb = viewport.window_bounds?;
    let w: f32 = wb.size.width.into();
    let h: f32 = wb.size.height.into();
    let corners = [
        viewport.screen_to_world(Point::new(px(0.0), px(0.0))),
        viewport.screen_to_world(Point::new(px(w), px(0.0))),
        viewport.screen_to_world(Point::new(px(w), px(h))),
        viewport.screen_to_world(Point::new(px(0.0), px(h))),
    ];
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for c in corners {
        let x: f32 = c.x.into();
        let y: f32 = c.y.into();
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    Some((min_x, min_y, max_x.max(min_x + 1.0), max_y.max(min_y + 1.0)))
}

fn node_edges(n: &Node) -> (f32, f32, f32, f32) {
    let x: f32 = n.x.into();
    let y: f32 = n.y.into();
    let w: f32 = n.size.width.into();
    let h: f32 = n.size.height.into();
    (x, y, x + w, y + h)
}

fn union_bbox(graph: &Graph, ids: &HashSet<NodeId>) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    let mut any = false;
    for id in ids {
        let n = graph.nodes.get(id)?;
        let (l, t, r, b) = node_edges(n);
        min_x = min_x.min(l);
        min_y = min_y.min(t);
        max_x = max_x.max(r);
        max_y = max_y.max(b);
        any = true;
    }
    any.then_some((min_x, min_y, max_x, max_y))
}

fn dedup_axis(v: &mut Vec<f32>) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v.dedup_by(|a, b| (*a - *b).abs() < 0.25);
}

pub(crate) fn compute_alignment_guides(
    graph: &Graph,
    viewport: &Viewport,
    dragged_ids: &HashSet<NodeId>,
) -> AlignmentGuides {
    if dragged_ids.is_empty() {
        return AlignmentGuides::default();
    }

    let Some((wx0, wy0, wx1, wy1)) = visible_world_bounds(viewport) else {
        return AlignmentGuides::default();
    };

    let Some(db) = union_bbox(graph, dragged_ids) else {
        return AlignmentGuides::default();
    };

    let thr = SNAP_SCREEN_PX / viewport.zoom.max(0.01);

    let mut ref_x: Vec<f32> = Vec::new();
    let mut ref_y: Vec<f32> = Vec::new();
    for (id, n) in graph.nodes() {
        if dragged_ids.contains(id) {
            continue;
        }
        let (l, t, r, b) = node_edges(n);
        let cx = (l + r) * 0.5;
        let cy = (t + b) * 0.5;
        ref_x.extend([l, cx, r]);
        ref_y.extend([t, cy, b]);
    }

    dedup_axis(&mut ref_x);
    dedup_axis(&mut ref_y);

    let (dl, dt, dr, db) = db;
    let dcx = (dl + dr) * 0.5;
    let dcy = (dt + db) * 0.5;

    let mut vertical_candidates: Vec<f32> = Vec::new();
    let xs = [dl, dcx, dr];
    for rx in ref_x {
        for dx in xs {
            if (dx - rx).abs() <= thr {
                vertical_candidates.push((dx + rx) * 0.5);
            }
        }
    }
    dedup_axis(&mut vertical_candidates);

    let mut horizontal_candidates: Vec<f32> = Vec::new();
    let ys = [dt, dcy, db];
    for ry in ref_y {
        for dy in ys {
            if (dy - ry).abs() <= thr {
                horizontal_candidates.push((dy + ry) * 0.5);
            }
        }
    }
    dedup_axis(&mut horizontal_candidates);

    let _ = (wx0, wy0, wx1, wy1);

    AlignmentGuides {
        vertical_x: vertical_candidates,
        horizontal_y: horizontal_candidates,
    }
}

/// While dragging nodes, draws magenta alignment lines when selection bbox matches other nodes.
pub struct SnapGuidesPlugin;

impl SnapGuidesPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for SnapGuidesPlugin {
    fn name(&self) -> &'static str {
        "snap_guides"
    }

    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}

    fn priority(&self) -> i32 {
        118
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Interaction
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let guides = ctx.alignment_guides.as_ref()?;
        if guides.vertical_x.is_empty() && guides.horizontal_y.is_empty() {
            return None;
        }

        let vxs = guides.vertical_x.clone();
        let hys = guides.horizontal_y.clone();
        let vp = ctx.viewport.clone();
        let (wx0, wy0, wx1, wy1) = visible_world_bounds(&vp)?;

        Some(
            canvas(
                move |_, _, _| (),
                move |_, _, win, _| {
                    for x in &vxs {
                        let a = vp.world_to_screen(Point::new(px(*x), px(wy0)));
                        let b = vp.world_to_screen(Point::new(px(*x), px(wy1)));
                        let mut line = PathBuilder::stroke(px(1.0));
                        line.move_to(a);
                        line.line_to(b);
                        if let Ok(p) = line.build() {
                            win.paint_path(p, rgb(GUIDE_COLOR));
                        }
                    }
                    for y in &hys {
                        let a = vp.world_to_screen(Point::new(px(wx0), px(*y)));
                        let b = vp.world_to_screen(Point::new(px(wx1), px(*y)));
                        let mut line = PathBuilder::stroke(px(1.0));
                        line.move_to(a);
                        line.line_to(b);
                        if let Ok(p) = line.build() {
                            win.paint_path(p, rgb(GUIDE_COLOR));
                        }
                    }
                },
            )
            .into_any(),
        )
    }
}
