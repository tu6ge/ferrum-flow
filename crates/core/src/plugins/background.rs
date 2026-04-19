use gpui::{
    BorderStyle, Bounds, Element as _, InteractiveElement as _, PaintQuad, ParentElement, Point,
    Size, Styled, canvas, div, px,
};

use crate::plugin::Plugin;

pub struct BackgroundPlugin;

impl BackgroundPlugin {
    pub fn new() -> Self {
        Self {}
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
        let zoom = ctx.zoom();
        let offset = ctx.offset();
        let bounds = ctx.window.bounds();
        let theme = ctx.theme.clone();
        let grid = 40.0_f32 * zoom;

        let el = canvas(
            move |_bounds, _window, _cx| (),
            move |_bounds, _state, window, _cx| {
                let width = f32::from(bounds.size.width);
                let height = f32::from(bounds.size.height);

                let start_x = f32::from(offset.x) % grid;
                let start_y = f32::from(offset.y) % grid;

                let dot_size = px(2.0);
                let dot_radius = px(1.0);

                let mut x = start_x;
                while x < width {
                    let mut y = start_y;
                    while y < height {
                        window.paint_quad(PaintQuad {
                            bounds: Bounds {
                                origin: Point::new(px(x - 1.0), px(y - 1.0)),
                                size: Size::new(dot_size, dot_size),
                            },
                            corner_radii: gpui::Corners::all(dot_radius),
                            background: gpui::rgb(theme.background_grid_dot).into(),
                            border_widths: gpui::Edges::all(px(0.0)),
                            border_color: gpui::transparent_black(),
                            border_style: BorderStyle::Solid,
                        });
                        y += grid;
                    }
                    x += grid;
                }
            },
        )
        .absolute()
        .size_full();

        Some(
            div()
                .id("background")
                .absolute()
                .size_full()
                .bg(gpui::rgb(ctx.theme.background))
                .child(el)
                .into_any(),
        )
    }
}
