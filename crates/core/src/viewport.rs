use gpui::{Bounds, Pixels, Point, Size, Window, px};

use crate::Node;

#[derive(Debug, Clone)]
pub struct Viewport {
    pub zoom: f32,
    pub offset: Point<Pixels>,
    pub window_bounds: Option<Bounds<Pixels>>,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            offset: Point::new(px(0.0), px(0.0)),
            window_bounds: None,
        }
    }

    /// Sets [`Self::window_bounds`] to the window’s drawable area (`Window::viewport_size`),
    /// origin `(0, 0)`. Skips assignment when width/height are unchanged.
    ///
    /// Prefer this over `Window::bounds()` for hit-testing and overlay layout: the latter is in
    /// global space and can be larger than the content viewport.
    pub fn sync_drawable_bounds(&mut self, window: &Window) {
        let vs = window.viewport_size();
        let unchanged = self.window_bounds.is_some_and(|b| {
            b.size.width == vs.width && b.size.height == vs.height
        });
        if !unchanged {
            self.window_bounds = Some(Bounds::new(
                Point::new(px(0.0), px(0.0)),
                Size::new(vs.width, vs.height),
            ));
        }
    }

    pub fn world_to_screen(&self, p: Point<Pixels>) -> Point<Pixels> {
        Point::new(
            p.x * self.zoom + self.offset.x,
            p.y * self.zoom + self.offset.y,
        )
    }

    pub fn screen_to_world(&self, p: Point<Pixels>) -> Point<Pixels> {
        Point::new(
            (p.x - self.offset.x) / self.zoom,
            (p.y - self.offset.y) / self.zoom,
        )
    }

    pub fn is_node_visible(&self, node: &Node) -> bool {
        let Some(window_bounds) = self.window_bounds else {
            return false;
        };

        let screen = self.world_to_screen(node.point());

        screen.x + node.size.width * self.zoom > px(0.0)
            && screen.x < window_bounds.size.width
            && screen.y + node.size.height * self.zoom > px(0.0)
            && screen.y < window_bounds.size.height
    }
}
