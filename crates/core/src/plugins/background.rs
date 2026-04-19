use std::sync::Arc;

use gpui::{
    App, Bounds, Corners, Element, ElementId, GlobalElementId, InspectorElementId,
    InteractiveElement as _, IntoElement, LayoutId, ParentElement, RenderImage, Style, Styled,
    Window, div, relative,
};
use image::{Frame, RgbaImage};
use smallvec::smallvec;

use crate::plugin::Plugin;

const BASE_GRID: f32 = 40.0;

fn rgb_components(c: u32) -> (u8, u8, u8) {
    let r = ((c >> 16) & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = (c & 0xFF) as u8;
    (r, g, b)
}

/// Raster aligned to the same grid math as the old `paint_quad` loop; returns BGRA bytes in a [`RenderImage`].
fn rasterize_dot_grid(
    width: u32,
    height: u32,
    grid: f32,
    start_x: f32,
    start_y: f32,
    bg_color: u32,
    dot_color: u32,
) -> Option<Arc<RenderImage>> {
    if width == 0 || height == 0 || grid <= 0.0 {
        return None;
    }
    let w = width as usize;
    let h = height as usize;
    let (br, bg, bb) = rgb_components(bg_color);
    let (dr, dg, db) = rgb_components(dot_color);

    let mut data = vec![0u8; w * h * 4];
    for i in 0..w * h {
        let p = i * 4;
        data[p] = br;
        data[p + 1] = bg;
        data[p + 2] = bb;
        data[p + 3] = 255;
    }

    let wf = width as f32;
    let hf = height as f32;
    let mut x = start_x;
    while x < wf {
        let mut y = start_y;
        while y < hf {
            let lx = x - 1.0;
            let ly = y - 1.0;
            for dy in 0..2 {
                for dx in 0..2 {
                    let px = (lx + dx as f32).floor() as isize;
                    let py = (ly + dy as f32).floor() as isize;
                    if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                        let i = ((py as usize) * w + (px as usize)) * 4;
                        data[i] = dr;
                        data[i + 1] = dg;
                        data[i + 2] = db;
                        data[i + 3] = 255;
                    }
                }
            }
            y += grid;
        }
        x += grid;
    }

    // GPUI expects BGRA for this path (see gpui `img` loader).
    for chunk in data.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    let img = RgbaImage::from_raw(width, height, data)?;
    let frame = Frame::new(img);
    Some(Arc::new(RenderImage::new(smallvec![frame])))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BackgroundLayoutCacheKey {
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    height: f32,
    bg_color: u32,
    dot_color: u32,
}

/// Custom element: draws the cached grid as a single [`Window::paint_image`].
struct BackgroundDots {
    image: Arc<RenderImage>,
}

impl BackgroundDots {
    fn new(image: Arc<RenderImage>) -> Self {
        Self { image }
    }
}

struct BackgroundPaintState {
    bounds: Bounds<gpui::Pixels>,
    image: Arc<RenderImage>,
}

impl IntoElement for BackgroundDots {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for BackgroundDots {
    type RequestLayoutState = ();
    type PrepaintState = BackgroundPaintState;

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::Name("background-dots".into()))
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();
        let layout_id = window.request_layout(style, [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<gpui::Pixels>,
        _state: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        BackgroundPaintState {
            bounds,
            image: Arc::clone(&self.image),
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<gpui::Pixels>,
        _layout: &mut Self::RequestLayoutState,
        state: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let _ = window.paint_image(
            state.bounds,
            Corners::default(),
            Arc::clone(&state.image),
            0,
            false,
        );
    }
}

pub struct BackgroundPlugin {
    cache_key: Option<BackgroundLayoutCacheKey>,
    image: Option<Arc<RenderImage>>,
}

impl BackgroundPlugin {
    pub fn new() -> Self {
        Self {
            cache_key: None,
            image: None,
        }
    }

    fn sync_raster(&mut self, ctx: &crate::plugin::RenderContext) {
        let bounds = ctx.window.bounds();
        let w = f32::from(bounds.size.width);
        let h = f32::from(bounds.size.height);
        let zoom = ctx.zoom();
        let grid = BASE_GRID * zoom;
        let offset = ctx.offset();
        let offset_x: f32 = offset.x.into();
        let offset_y: f32 = offset.y.into();
        let start_x = offset_x % grid;
        let start_y = offset_y % grid;
        let bg = ctx.theme.background;
        let dot = ctx.theme.background_grid_dot;

        let key = BackgroundLayoutCacheKey {
            zoom,
            offset_x,
            offset_y,
            width: w,
            height: h,
            bg_color: bg,
            dot_color: dot,
        };

        if self.cache_key != Some(key) {
            self.cache_key = Some(key);
            let width_u = w.max(0.0) as u32;
            let height_u = h.max(0.0) as u32;
            self.image = rasterize_dot_grid(
                width_u,
                height_u,
                grid,
                start_x,
                start_y,
                bg,
                dot,
            );
        }
    }
}

impl Plugin for BackgroundPlugin {
    fn name(&self) -> &'static str {
        "background"
    }
    fn priority(&self) -> i32 {
        0
    }
    fn render_layer(&self) -> crate::plugin::RenderLayer {
        crate::plugin::RenderLayer::Background
    }
    fn render(&mut self, ctx: &mut crate::plugin::RenderContext) -> Option<gpui::AnyElement> {
        self.sync_raster(ctx);
        let Some(image) = self.image.as_ref() else {
            return Some(
                div()
                    .id("background")
                    .absolute()
                    .size_full()
                    .bg(gpui::rgb(ctx.theme.background))
                    .into_any(),
            );
        };

        Some(
            div()
                .id("background")
                .absolute()
                .size_full()
                .bg(gpui::rgb(ctx.theme.background))
                .child(BackgroundDots::new(Arc::clone(image)))
                .into_any(),
        )
    }
}
