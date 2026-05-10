//! Custom [`NodeRenderer`]: `7` / `8` stacked, `+` (two in, one out), and a **result** node showing `15`.
//!
//! Run: `cargo run -p ferrum-flow --example renderer`

use ferrum_flow::*;
use gpui::{
    AnyElement, AppContext as _, Application, Bounds, Element as _, ParentElement as _, Size,
    Styled, WindowBounds, WindowOptions, div, px, rgb,
};
use serde_json::json;

fn main() {
    Application::new().run(|cx| {
        let graph = Graph::build(|g| {
            let gap = 14.0;
            let digit_w = 68.0;
            let digit_h = 48.0;
            let col_x = 96.0;
            let y7 = 88.0;
            let y8 = y7 + digit_h + gap;

            let (_, _, outs_7) = g
                .create_node("calc")
                .position(col_x, y7)
                .size(digit_w, digit_h)
                .output()
                .data(json!({ "kind": "digit", "label": "7" }))
                .build_with_ports();

            let (_, _, outs_8) = g
                .create_node("calc")
                .position(col_x, y8)
                .size(digit_w, digit_h)
                .output()
                .data(json!({ "kind": "digit", "label": "8" }))
                .build_with_ports();

            let mid_y = (y7 + digit_h * 0.5 + y8 + digit_h * 0.5) * 0.5;
            let plus_h = 64.0;
            let plus_y = mid_y - plus_h * 0.5;

            let (_, ins_plus, outs_plus) = g
                .create_node("calc")
                .position(col_x + digit_w + 52.0, plus_y)
                .size(72.0, plus_h)
                .input()
                .input()
                .output()
                .data(json!({ "kind": "op", "label": "+" }))
                .build_with_ports();

            let (_, ins_result, _) = g
                .create_node("calc")
                .position(col_x + digit_w + 52.0 + 72.0 + 48.0, mid_y - 26.0)
                .size(92.0, 52.0)
                .input()
                .data(json!({ "kind": "result", "label": "15" }))
                .build_with_ports();

            g.create_edge()
                .source(outs_7[0])
                .target(ins_plus[0])
                .build();
            g.create_edge()
                .source(outs_8[0])
                .target(ins_plus[1])
                .build();
            g.create_edge()
                .source(outs_plus[0])
                .target(ins_result[0])
                .build();
        });

        // Smaller window so [`FitAllGraphPlugin`] picks a lower zoom — nodes don’t dominate the screen.
        let win_size = Size::new(px(480.0), px(360.0));
        let window_opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, win_size, cx))),
            ..Default::default()
        };

        cx.open_window(window_opts, |window, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .default_plugins()
                    .plugin(FitAllGraphPlugin::new())
                    .node_renderer("calc", CalculatorRenderer)
                    .build()
            })
        })
        .unwrap();
    });
}

pub struct CalculatorRenderer;

impl NodeRenderer for CalculatorRenderer {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement {
        let data = node.data_ref();
        let kind = data.get("kind").and_then(|v| v.as_str()).unwrap_or("digit");
        let label = data
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();

        let (bg, fg, border, text_size) = match kind {
            "op" => (rgb(0x0F766E), rgb(0xF0FDFA), rgb(0x0D9488), "lg"),
            "result" => (rgb(0x312E81), rgb(0xE0E7FF), rgb(0x4338CA), "xl"),
            // Distinct from canvas [`FlowTheme::default`].background (#F8F9FB): cool-tinted card.
            _ => (rgb(0xDBEAFE), rgb(0x1E3A8A), rgb(0x60A5FA), "lg"),
        };

        let shell = ctx
            .node_card_shell_custom(node)
            .rounded(px(8.0))
            .bg(bg)
            .border_1()
            .border_color(border);

        let text = div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .text_color(fg)
            .child(label);

        let text = match text_size {
            "xl" => text.text_xl(),
            _ => text.text_lg(),
        };

        shell.child(text).into_any()
    }

    fn port_render(&self, node: &Node, port: &Port, ctx: &mut RenderContext) -> Option<AnyElement> {
        let frame = ctx.port_screen_frame(node, port)?;
        let kind = node
            .data_ref()
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("digit");
        // Match each node kind so sockets don’t use theme default (dark disc + gray ring on pastel cards).
        let (fill, ring) = match (kind, port.kind()) {
            ("op", _) => (rgb(0xF0FDFA), rgb(0x14B8A6)),
            ("result", _) => (rgb(0xEEF2FF), rgb(0x818CF8)),
            (_, PortKind::Input) => (rgb(0xFFFFFF), rgb(0x2563EB)),
            (_, PortKind::Output) => (rgb(0xEFF6FF), rgb(0x3B82F6)),
        };
        Some(
            frame
                .anchor_div()
                .rounded_full()
                .border_1()
                .border_color(ring)
                .bg(fill)
                .into_any(),
        )
    }
}
