//! Top-left **sample graphs** dropdown (hit-testing like the zoom control bar).

use ferrum_flow::{
    EventResult, FlowEvent, InputEvent, Plugin, PluginContext, RenderContext, RenderLayer,
};
use gpui::{
    Bounds, FontWeight, IntoElement as _, MouseButton, ParentElement as _, Pixels, Point, Size,
    Styled as _, div, px, rgb, rgba,
};

use crate::demo_graph::SHADER_STUDIO_DEMOS;
use crate::graph_io::replace_canvas_graph;

const MARGIN_L: f32 = 14.0;
const MARGIN_T: f32 = 14.0;
const HEADER_H: f32 = 32.0;
const MENU_W: f32 = 216.0;
const ROW_H: f32 = 30.0;
const RADIUS: f32 = 8.0;
/// Gap between title row and panel (must match `.mt` below).
const GAP_HEADER_PANEL: f32 = 4.0;
/// Panel vertical padding (must match `.py`).
const PANEL_PAD_Y: f32 = 4.0;

#[derive(Clone)]
struct DemoMenuLayout {
    header: Bounds<Pixels>,
    rows: Vec<Bounds<Pixels>>,
}

impl DemoMenuLayout {
    fn hit_header(&self, p: Point<gpui::Pixels>) -> bool {
        self.header.contains(&p)
    }

    fn hit_row(&self, p: Point<gpui::Pixels>) -> Option<usize> {
        for (i, b) in self.rows.iter().enumerate() {
            if b.contains(&p) {
                return Some(i);
            }
        }
        None
    }

    fn any_hit(&self, p: Point<gpui::Pixels>) -> bool {
        self.hit_header(p) || self.hit_row(p).is_some()
    }
}

fn build_layout(menu_open: bool, win_w: f32, win_h: f32) -> Option<DemoMenuLayout> {
    if win_w < MENU_W + MARGIN_L + 8.0 || win_h < HEADER_H + MARGIN_T + 24.0 {
        return None;
    }
    let x0 = MARGIN_L;
    let y0 = MARGIN_T;
    let header = Bounds::new(
        Point::new(px(x0), px(y0)),
        Size::new(px(MENU_W), px(HEADER_H)),
    );
    let mut rows = Vec::new();
    if menu_open {
        let first_row_y = y0 + HEADER_H + GAP_HEADER_PANEL + PANEL_PAD_Y;
        for i in 0..SHADER_STUDIO_DEMOS.len() {
            let y = first_row_y + i as f32 * ROW_H;
            rows.push(Bounds::new(
                Point::new(px(x0), px(y)),
                Size::new(px(MENU_W), px(ROW_H)),
            ));
        }
    }
    Some(DemoMenuLayout { header, rows })
}

pub struct DemoMenuPlugin {
    menu_open: bool,
    last_layout: Option<DemoMenuLayout>,
}

impl DemoMenuPlugin {
    pub fn new() -> Self {
        Self {
            menu_open: false,
            last_layout: None,
        }
    }
}

impl Plugin for DemoMenuPlugin {
    fn name(&self) -> &'static str {
        "shader_demo_menu"
    }

    fn priority(&self) -> i32 {
        129
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        let FlowEvent::Input(InputEvent::MouseDown(ev)) = event else {
            return EventResult::Continue;
        };
        if ev.button != MouseButton::Left {
            return EventResult::Continue;
        }

        let Some(ref layout) = self.last_layout else {
            return EventResult::Continue;
        };

        let p = ev.position;

        if layout.hit_header(p) {
            self.menu_open = !self.menu_open;
            ctx.notify();
            return EventResult::Stop;
        }

        if self.menu_open {
            if let Some(i) = layout.hit_row(p) {
                let (_t, g) = crate::demo_graph::shader_demo_select(i);
                replace_canvas_graph(ctx, g);
                self.menu_open = false;
                ctx.notify();
                return EventResult::Stop;
            }
            if !layout.any_hit(p) {
                self.menu_open = false;
                ctx.notify();
                return EventResult::Stop;
            }
        }

        EventResult::Continue
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let win = ctx.window_bounds().unwrap_or_else(|| {
            let vs = ctx.window.viewport_size();
            Bounds::new(Point::new(px(0.0), px(0.0)), Size::new(vs.width, vs.height))
        });
        let ww: f32 = win.size.width.into();
        let wh: f32 = win.size.height.into();

        let Some(layout) = build_layout(self.menu_open, ww, wh) else {
            self.last_layout = None;
            return None;
        };
        self.last_layout = Some(layout.clone());

        let t = ctx.theme;
        let bg = rgba(t.context_menu_background << 8 | 0xe8);
        let border = rgb(t.context_menu_border);
        let fg = rgb(t.context_menu_text);
        let chevron = if self.menu_open { "▴" } else { "▾" };

        let header_el = div()
            .w(px(MENU_W))
            .h(px(HEADER_H))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px(px(10.0))
            .rounded(px(RADIUS))
            .border_1()
            .border_color(border)
            .bg(bg)
            .shadow_sm()
            .text_sm()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(fg)
            .child("Samples")
            .child(
                div()
                    .text_color(rgb(t.context_menu_shortcut_text))
                    .child(chevron.to_string()),
            );

        let mut col = vec![header_el.into_any_element()];

        if self.menu_open {
            let panel = div()
                .mt(px(GAP_HEADER_PANEL))
                .w(px(MENU_W))
                .rounded(px(RADIUS))
                .border_1()
                .border_color(border)
                .bg(bg)
                .shadow_sm()
                .flex()
                .flex_col()
                .py(px(PANEL_PAD_Y));

            let mut panel = panel;
            for (name, _) in SHADER_STUDIO_DEMOS {
                let row = div()
                    .w(px(MENU_W))
                    .h(px(ROW_H))
                    .px(px(10.0))
                    .flex()
                    .items_center()
                    .text_sm()
                    .text_color(fg)
                    .child(format!("· {name}"));
                panel = panel.child(row);
            }
            col.push(panel.into_any_element());
        }

        Some(
            div()
                .absolute()
                .size_full()
                .child(
                    div()
                        .absolute()
                        .top(px(MARGIN_T))
                        .left(px(MARGIN_L))
                        .w(px(MENU_W))
                        .flex()
                        .flex_col()
                        .children(col),
                )
                .into_any_element(),
        )
    }
}
