//! Custom cards for AI agent workflows — reads optional `node.data.title` / `subtitle`.

use ferrum_flow::{Node, NodeRenderer, Port, PortKind, RenderContext};
use gpui::{AnyElement, Element as _, ParentElement as _, Styled, div, px, rgb, rgba};
use serde_json::Value;

use crate::theme::{
    PORT_IN, PORT_OUT, PORT_RING, accent_agent, accent_io_in, accent_io_out, accent_llm,
    accent_router, accent_tool,
};

#[derive(Clone, Copy)]
pub enum WorkflowKind {
    IoStart,
    IoEnd,
    Agent,
    Llm,
    Tool,
    Router,
    /// Default for `node_type == ""` (e.g. quick-add from port dot).
    Step,
}

impl WorkflowKind {
    fn accent(self) -> u32 {
        match self {
            Self::IoStart => accent_io_in(),
            Self::IoEnd => accent_io_out(),
            Self::Agent => accent_agent(),
            Self::Llm => accent_llm(),
            Self::Tool => accent_tool(),
            Self::Router => accent_router(),
            Self::Step => 0x90a4ae,
        }
    }

    fn default_title(self) -> &'static str {
        match self {
            Self::IoStart => "Input",
            Self::IoEnd => "Output",
            Self::Agent => "Agent",
            Self::Llm => "Model",
            Self::Tool => "Tool",
            Self::Router => "Router",
            Self::Step => "New step",
        }
    }

    fn default_subtitle(self) -> Option<&'static str> {
        match self {
            Self::Agent => Some("Plan · act · observe"),
            Self::Llm => Some("Chat completion"),
            Self::Tool => Some("Function call"),
            Self::Router => Some("Branch logic"),
            Self::IoStart => Some("User / trigger"),
            Self::IoEnd => Some("Final answer"),
            Self::Step => Some("Rename in data · choose type"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct WorkflowNodeRenderer {
    pub kind: WorkflowKind,
}

impl WorkflowNodeRenderer {
    pub const fn new(kind: WorkflowKind) -> Self {
        Self { kind }
    }

    fn read_title<'a>(&self, node: &'a Node) -> String {
        data_str(&node.data_ref(), "title")
            .map(String::from)
            .unwrap_or_else(|| self.kind.default_title().to_string())
    }

    fn read_subtitle(&self, node: &Node) -> Option<String> {
        data_str(&node.data_ref(), "subtitle")
            .map(String::from)
            .or_else(|| self.kind.default_subtitle().map(String::from))
    }

    fn card_bg(&self) -> u32 {
        match self.kind {
            WorkflowKind::IoStart => 0x121c14,
            WorkflowKind::IoEnd => 0x1c1414,
            WorkflowKind::Agent => 0x1a1628,
            WorkflowKind::Llm => 0x101c28,
            WorkflowKind::Tool => 0x101c18,
            WorkflowKind::Router => 0x1c1a12,
            WorkflowKind::Step => 0x161c22,
        }
    }
}

fn data_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
    data.get(key).and_then(|v| v.as_str())
}

impl NodeRenderer for WorkflowNodeRenderer {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement {
        let node_id = node.id();
        let selected = ctx.graph.selected_node().contains(&node_id);
        let screen = ctx.world_to_screen(node.point());
        let w = ctx.world_length_to_screen(node.size_ref().width);
        let h = ctx.world_length_to_screen(node.size_ref().height);
        let accent = self.kind.accent();
        let bg = self.card_bg();
        let title = self.read_title(node);
        let subtitle = self.read_subtitle(node);

        let border = if selected {
            rgb(ctx.theme.selection_rect_border)
        } else {
            // 0xRRGGBBAA — soft accent rim
            rgba((accent << 8) | 0x55)
        };

        let header = div().w_full().h(px(3.0)).bg(rgb(accent));

        let mut body = div().flex().flex_col().gap(px(4.0)).p(px(10.0)).child(
            div()
                .child(title)
                .text_color(rgb(ctx.theme.node_caption_text))
                .text_size(px(13.0)),
        );

        if let Some(sub) = subtitle {
            if !sub.is_empty() {
                body = body.child(
                    div()
                        .child(sub)
                        .text_color(rgb(ctx.theme.undefined_node_caption_text))
                        .text_size(px(11.0)),
                );
            }
        }

        div()
            .absolute()
            .left(screen.x)
            .top(screen.y)
            .w(w)
            .h(h)
            .rounded(px(12.0))
            .overflow_hidden()
            .border(px(if selected { 2.0 } else { 1.0 }))
            .border_color(border)
            .shadow_md()
            .bg(rgb(bg))
            .child(header)
            .child(body.flex_1())
            .into_any()
    }

    fn port_render(&self, node: &Node, port: &Port, ctx: &mut RenderContext) -> Option<AnyElement> {
        let frame = ctx.port_screen_frame(node, port)?;
        let z = frame.zoom;
        let size = frame.size;
        let o = frame.origin();

        let (ring, core) = match port.kind() {
            PortKind::Input => (rgb(PORT_IN), rgb(PORT_RING)),
            PortKind::Output => (rgb(PORT_OUT), rgb(PORT_RING)),
        };

        Some(
            div()
                .absolute()
                .left(o.x)
                .top(o.y)
                .w(size.width * z + px(4.0))
                .h(size.height * z + px(4.0))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .w(size.width * z)
                        .h(size.height * z)
                        .rounded_full()
                        .border(px(2.0))
                        .border_color(ring)
                        .bg(core),
                )
                .into_any(),
        )
    }
}
