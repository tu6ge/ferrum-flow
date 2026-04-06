//! Offscreen wgpu: run WGSL from the graph and show it as a GPUI image on the canvas.

use std::sync::Arc;
use std::time::{Duration, Instant};

use ferrum_flow::{
    EventResult, FlowEvent, Plugin, RenderContext, RenderLayer,
};
use gpui::{
    Bounds, FontWeight, ImageSource, IntoElement as _, ParentElement as _, Point, RenderImage, Size,
    Styled as _, div, img, px, rgb, rgba,
};
use image::{Frame, RgbaImage};
use smallvec::SmallVec;

use crate::compile_graph_to_wgsl;
use crate::preview_rig::PreviewRig;
use crate::wgsl_preview::graph_fingerprint;

const PREVIEW_W: u32 = 320;
const PREVIEW_H: u32 = 180;
const FRAME_INTERVAL: Duration = Duration::from_millis(48);
const RIG_RETRY: Duration = Duration::from_millis(400);

fn rgba_to_gpui_frame(mut rgba: Vec<u8>, width: u32, height: u32) -> Frame {
    for px in rgba.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    let buf =
        RgbaImage::from_raw(width, height, rgba).expect("buffer size matches width*height*4");
    Frame::new(buf)
}

pub struct WgpuPreviewPlugin {
    graph_fp: u64,
    wgsl_source: String,
    rig: Option<PreviewRig>,
    compile_ok: bool,
    err_short: String,
    last_frame: Option<Arc<RenderImage>>,
    last_frame_at: Option<Instant>,
    last_rig_attempt: Option<Instant>,
    started: Instant,
    last_gpu_err: Option<String>,
}

impl WgpuPreviewPlugin {
    pub fn new() -> Self {
        Self {
            graph_fp: 0,
            wgsl_source: String::new(),
            rig: None,
            compile_ok: false,
            err_short: String::new(),
            last_frame: None,
            last_frame_at: None,
            last_rig_attempt: None,
            started: Instant::now(),
            last_gpu_err: None,
        }
    }

    fn rebuild_rig(&mut self) {
        match PreviewRig::new(&self.wgsl_source, PREVIEW_W, PREVIEW_H) {
            Ok(r) => {
                self.rig = Some(r);
                self.last_gpu_err = None;
                self.last_frame = None;
                self.last_frame_at = None;
            }
            Err(e) => {
                self.last_gpu_err = Some(e.clone());
                self.rig = None;
            }
        }
    }

    fn sync_graph(&mut self, g: &ferrum_flow::Graph) {
        let gfp = graph_fingerprint(g);
        if gfp != self.graph_fp {
            self.graph_fp = gfp;
            self.last_gpu_err = None;
            self.last_rig_attempt = None;
            self.wgsl_source.clear();
            match compile_graph_to_wgsl(g) {
                Ok(wgsl) => {
                    self.compile_ok = true;
                    self.err_short.clear();
                    self.wgsl_source = wgsl;
                    self.rebuild_rig();
                }
                Err(e) => {
                    let msg = e.to_string();
                    self.compile_ok = false;
                    self.err_short = if msg.chars().count() > 200 {
                        format!("{}…", msg.chars().take(200).collect::<String>())
                    } else {
                        msg
                    };
                    self.rig = None;
                    self.last_frame = None;
                    self.last_frame_at = None;
                }
            }
            return;
        }

        if self.compile_ok && self.rig.is_none() && !self.wgsl_source.is_empty() {
            let now = Instant::now();
            if self.last_rig_attempt.map_or(true, |t| {
                now.saturating_duration_since(t) > RIG_RETRY
            }) {
                self.last_rig_attempt = Some(now);
                self.rebuild_rig();
            }
        }
    }

    fn maybe_render(&mut self) -> Option<Arc<RenderImage>> {
        let rig = self.rig.as_mut()?;
        let now = Instant::now();
        if let Some(prev) = self.last_frame_at {
            if now.saturating_duration_since(prev) < FRAME_INTERVAL {
                return self.last_frame.clone();
            }
        }
        let t = self.started.elapsed().as_secs_f32();
        match rig.render_frame(t) {
            Ok(rgba) => {
                self.last_gpu_err = None;
                let frame = rgba_to_gpui_frame(rgba, PREVIEW_W, PREVIEW_H);
                let image = Arc::new(RenderImage::new(SmallVec::from_elem(frame, 1)));
                self.last_frame = Some(image.clone());
                self.last_frame_at = Some(now);
                Some(image)
            }
            Err(e) => {
                self.last_gpu_err = Some(e.clone());
                self.last_frame = None;
                self.last_frame_at = None;
                None
            }
        }
    }
}

impl Plugin for WgpuPreviewPlugin {
    fn name(&self) -> &'static str {
        "wgpu_preview"
    }

    fn priority(&self) -> i32 {
        126
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    fn on_event(&mut self, _event: &FlowEvent, _ctx: &mut ferrum_flow::PluginContext) -> EventResult {
        EventResult::Continue
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        self.sync_graph(ctx.graph);

        let win = ctx.viewport.window_bounds.unwrap_or_else(|| {
            let vs = ctx.window.viewport_size();
            Bounds::new(Point::new(px(0.0), px(0.0)), Size::new(vs.width, vs.height))
        });
        let wh: f32 = win.size.height.into();
        let ww: f32 = win.size.width.into();
        if wh < 200.0 || ww < 280.0 {
            return None;
        }

        let panel_w = (ww * 0.42).clamp(260.0, 520.0);
        let panel_h = (wh * 0.44).clamp(160.0, wh - 100.0);
        let gap = 12.0;
        let preview_top = 56.0 + panel_h + gap;

        let title = if !self.compile_ok {
            "GPU preview · compile failed"
        } else if self.last_gpu_err.is_some() {
            "GPU preview · device error"
        } else {
            "GPU preview · wgpu live"
        };

        let border = if self.compile_ok && self.last_gpu_err.is_none() {
            0x002a6b66_u32
        } else {
            0x00aa5533_u32
        };

        let header = div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(ctx.theme.context_menu_text))
            .mb(px(6.0))
            .child(title.to_string());

        let body: gpui::AnyElement = if let Some(ref e) = self.last_gpu_err {
            div()
                .text_xs()
                .text_color(rgb(ctx.theme.context_menu_shortcut_text))
                .child(e.clone())
                .into_any_element()
        } else if !self.compile_ok {
            div()
                .text_xs()
                .text_color(rgb(ctx.theme.context_menu_shortcut_text))
                .child(self.err_short.clone())
                .into_any_element()
        } else if let Some(img_src) = self.maybe_render().map(ImageSource::Render) {
            img(img_src)
                .w(px(PREVIEW_W as f32))
                .h(px(PREVIEW_H as f32))
                .into_any_element()
        } else if self.rig.is_some() {
            div()
                .text_xs()
                .text_color(rgb(ctx.theme.context_menu_shortcut_text))
                .child("Rendering…")
                .into_any_element()
        } else {
            div()
                .text_xs()
                .text_color(rgb(ctx.theme.context_menu_shortcut_text))
                .child("No pipeline available")
                .into_any_element()
        };

        Some(
            div()
                .absolute()
                .size_full()
                .child(
                    div()
                        .absolute()
                        .top(px(preview_top))
                        .right(px(14.0))
                        .w(px(panel_w))
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(rgb(border))
                        .bg(rgba(0x0a1020dd))
                        .shadow_sm()
                        .flex()
                        .flex_col()
                        .p(px(10.0))
                        .child(header)
                        .child(body),
                )
                .into_any_element(),
        )
    }
}
