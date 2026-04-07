use ferrum_flow::{
    Node, NodeCardVariant, NodeRenderer, Port, RenderContext, default_node_caption,
};
use gpui::{
    AnyElement, Element, FontWeight, ParentElement, Styled, div, prelude::FluentBuilder, px, rgb,
    rgba,
};

#[derive(Clone, Copy)]
pub struct ShaderNodeRenderer;

fn card_colors(node_type: &str) -> (u32, u32) {
    match node_type {
        "uv" | "scalar" => (0x001a3d42, 0x002dd4bf),
        "time" => (0x003d2f1a, 0x00ffb86b),
        "join_ff" | "sub_vec2" | "length_v2" | "sin_f" | "mul_ff" | "add_ff" | "mul_vec2_f"
        | "smoothstep" => (0x0021324a, 0x00a8c5ff),
        "noise" => (0x002d1f3d, 0x00ffca28),
        "color" | "mix" => (0x003d2818, 0x00ff79c6),
        "output" => (0x0018253d, 0x007dd3fc),
        _ => (0x00252830, 0x0049505a),
    }
}

fn category_badge(node_type: &str) -> Option<(&'static str, u32)> {
    match node_type {
        "uv" | "time" | "scalar" => Some(("INPUT", 0x002dd4bf)),
        "join_ff" | "sub_vec2" | "length_v2" | "sin_f" | "mul_ff" | "add_ff" | "mul_vec2_f"
        | "smoothstep" => Some(("MATH", 0x00a8c5ff)),
        "noise" => Some(("PROC", 0x00ffca28)),
        "color" | "mix" => Some(("COLOR", 0x00ff79c6)),
        "output" => Some(("OUT", 0x007dd3fc)),
        _ => None,
    }
}

fn caption_secondary(node: &Node) -> Option<String> {
    node.data
        .get("hint")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

impl NodeRenderer for ShaderNodeRenderer {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement {
        let node_id = node.id;
        let selected = ctx.graph.selected_node.iter().any(|id| *id == node_id);

        let (bg, border) = card_colors(node.node_type.as_str());
        let border = if selected {
            ctx.theme.node_card_border_selected
        } else {
            border
        };

        let title = default_node_caption(node);
        let hint = caption_secondary(node);
        let badge = category_badge(node.node_type.as_str());

        let badge_el = badge.map(|(label, accent)| {
            div()
                .px(px(6.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .bg(rgba(0x00000055))
                .border_1()
                .border_color(rgb(accent))
                .text_color(rgb(accent))
                .child(label)
        });

        ctx.node_card_shell(node, selected, NodeCardVariant::Custom)
            .bg(rgb(bg))
            .border_color(rgb(border))
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(4.0))
                    .px_2()
                    .when_some(badge_el, |this, b| {
                        this.child(
                            div()
                                .w_full()
                                .flex()
                                .justify_start()
                                .child(b),
                        )
                    })
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(title)
                            .text_color(rgb(ctx.theme.node_caption_text)),
                    )
                    .when_some(hint, |this, h| {
                        this.child(
                            div()
                                .text_xs()
                                .opacity(0.72)
                                .text_center()
                                .child(h)
                                .text_color(rgb(ctx.theme.node_caption_text)),
                        )
                    }),
            )
            .into_any()
    }

    fn port_render(&self, node: &Node, port: &Port, ctx: &mut RenderContext) -> Option<AnyElement> {
        let (_, accent) = card_colors(node.node_type.as_str());
        let frame = ctx.port_screen_frame(node, port)?;

        Some(
            frame
                .anchor_div()
                .rounded_full()
                .border_1()
                .border_color(rgb(accent))
                .bg(rgba(0x00000088))
                .into_any(),
        )
    }
}
