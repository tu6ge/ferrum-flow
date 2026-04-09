//! Top-right **live WGSL preview**: re-runs `compile_graph_to_wgsl` when the graph changes; copyable text.

use std::hash::{Hash, Hasher};

use ferrum_flow::{
    EventResult, FlowEvent, Plugin, RenderContext, RenderLayer,
};
use gpui::{
    Bounds, FontWeight, IntoElement as _, ParentElement as _, Point, Size, Styled as _, div, px,
    rgb, rgba,
};

use crate::compile_graph_to_wgsl;

/// Fingerprint for graph structure (counts, node types/positions, edges); shared with GPU preview.
pub(crate) fn graph_fingerprint(g: &ferrum_flow::Graph) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();
    g.nodes().len().hash(&mut h);
    g.edges().len().hash(&mut h);
    for id in g.node_order() {
        let Some(n) = g.get_node(id) else {
            continue;
        };
        n.node_type.hash(&mut h);
        let xf: f32 = n.x.into();
        let yf: f32 = n.y.into();
        xf.to_bits().hash(&mut h);
        yf.to_bits().hash(&mut h);
    }
    for e in g.edges_values() {
        e.source_port.hash(&mut h);
        e.target_port.hash(&mut h);
    }
    h.finish()
}

pub struct WgslPreviewPlugin {
    fingerprint: u64,
    title: String,
    body: String,
    ok: bool,
}

fn truncate_chars(s: String, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s;
    }
    let head: String = s.chars().take(max_chars).collect();
    format!("{head}\n\n… (truncated; use compile_graph_to_wgsl for the full shader)")
}

impl WgslPreviewPlugin {
    pub fn new() -> Self {
        Self {
            fingerprint: 0,
            title: "WGSL preview".to_string(),
            body: String::new(),
            ok: true,
        }
    }

    fn refresh(&mut self, g: &ferrum_flow::Graph) {
        let fp = graph_fingerprint(g);
        if fp == self.fingerprint && !self.body.is_empty() {
            return;
        }
        self.fingerprint = fp;
        const MAX_CHARS: usize = 14_000;
        match compile_graph_to_wgsl(g) {
            Ok(w) => {
                self.ok = true;
                self.title = "WGSL preview · generated from graph (copyable)".to_string();
                self.body = truncate_chars(w, MAX_CHARS);
            }
            Err(e) => {
                self.ok = false;
                self.title = "WGSL compile failed".to_string();
                self.body = truncate_chars(e.to_string(), MAX_CHARS);
            }
        }
    }
}

impl Plugin for WgslPreviewPlugin {
    fn name(&self) -> &'static str {
        "wgsl_preview"
    }

    fn priority(&self) -> i32 {
        127
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    fn on_event(&mut self, _event: &FlowEvent, _ctx: &mut ferrum_flow::PluginContext) -> EventResult {
        EventResult::Continue
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        self.refresh(ctx.graph);

        let win = ctx.viewport.window_bounds().unwrap_or_else(|| {
            let vs = ctx.window.viewport_size();
            Bounds::new(Point::new(px(0.0), px(0.0)), Size::new(vs.width, vs.height))
        });
        let wh: f32 = win.size.height.into();
        let ww: f32 = win.size.width.into();
        if wh < 120.0 || ww < 280.0 {
            return None;
        }

        let panel_w = (ww * 0.42).clamp(260.0, 520.0);
        let panel_h = (wh * 0.44).clamp(160.0, wh - 100.0);

        let border = if self.ok {
            0x003d5c80_u32
        } else {
            0x00cc4444_u32
        };
        let fg = ctx.theme.context_menu_text;
        let sub = ctx.theme.context_menu_shortcut_text;

        let header = div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(fg))
            .mb(px(6.0))
            .child(self.title.clone());

        let code = div()
            .flex_1()
            .min_h(px(0.0))
            .overflow_hidden()
            .font_family("SF Mono")
            .text_xs()
            .text_color(rgb(sub))
            .child(self.body.clone());

        Some(
            div()
                .absolute()
                .size_full()
                .child(
                    div()
                        .absolute()
                        .top(px(56.0))
                        .right(px(14.0))
                        .w(px(panel_w))
                        .h(px(panel_h))
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(rgb(border))
                        .bg(rgba(0x0a1020dd))
                        .shadow_sm()
                        .flex()
                        .flex_col()
                        .p(px(10.0))
                        .child(header)
                        .child(code),
                )
                .into_any_element(),
        )
    }
}
