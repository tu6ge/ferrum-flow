use gpui::*;
use std::collections::HashMap;

use crate::node::Node;
use crate::plugin::RenderContext;
use crate::{Graph, Port, PortPosition};

pub trait NodeRenderer: Send + Sync {
    /// render node inner UI
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement;

    fn port_offset(&self, node: &Node, port: &Port, graph: &Graph) -> Point<Pixels> {
        let ports: Vec<&Port> = graph
            .ports
            .values()
            .filter(|p| p.node_id == node.id && p.kind == port.kind && p.position == port.position)
            .collect();

        let total = ports.len() as f32;
        let index = port.index as f32;
        let size = node.size;

        match port.position {
            PortPosition::Left => {
                let spacing = size.height / (total + 1.0);
                Point::new(px(0.0), spacing * (index + 1.0))
            }
            PortPosition::Right => {
                let spacing = size.height / (total + 1.0);
                Point::new(size.width, spacing * (index + 1.0))
            }
            PortPosition::Top => {
                let spacing = size.width / (total + 1.0);
                Point::new(spacing * (index + 1.0), px(0.0))
            }
            PortPosition::Bottom => {
                let spacing = size.width / (total + 1.0);
                Point::new(spacing * (index + 1.0), size.height)
            }
        }
    }
}

pub struct RendererRegistry {
    map: HashMap<String, Box<dyn NodeRenderer>>,
    default: Box<dyn NodeRenderer>,
    undefined: Box<dyn NodeRenderer>,
}

impl RendererRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            default: Box::new(DefaultNodeRenderer {}),
            undefined: Box::new(UndefinedNodeRenderer {}),
        }
    }

    pub fn register<R>(&mut self, name: impl Into<String>, renderer: R)
    where
        R: NodeRenderer + 'static,
    {
        self.map.insert(name.into(), Box::new(renderer));
    }

    pub fn get(&self, name: &str) -> &dyn NodeRenderer {
        if name.is_empty() {
            return self.default.as_ref();
        }

        self.map
            .get(name)
            .map(|r| r.as_ref())
            .unwrap_or(self.undefined.as_ref())
    }
}

struct DefaultNodeRenderer;

impl NodeRenderer for DefaultNodeRenderer {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement {
        let node_id = node.id;
        let screen = ctx.world_to_screen(node.point());
        let node_x = screen.x;
        let node_y = screen.y;
        let selected = ctx
            .graph
            .selected_node
            .iter()
            .find(|id| **id == node_id)
            .is_some();

        div()
            .absolute()
            .left(node_x)
            .top(node_y)
            .w(node.size.width * ctx.viewport.zoom)
            .h(node.size.height * ctx.viewport.zoom)
            .bg(white())
            .rounded(px(6.0))
            .border(px(1.5))
            .border_color(rgb(if selected { 0xFF7800 } else { 0x1A192B }))
            .child(
                div()
                    .child(format!("Node {}", node_id))
                    .text_color(rgb(0x1A192B)),
            )
            .into_any()
    }
}

struct UndefinedNodeRenderer;

impl NodeRenderer for UndefinedNodeRenderer {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement {
        let screen = ctx.world_to_screen(node.point());
        let node_x = screen.x;
        let node_y = screen.y;

        div()
            .absolute()
            .left(node_x)
            .top(node_y)
            .w(node.size.width * ctx.viewport.zoom)
            .h(node.size.height * ctx.viewport.zoom)
            .bg(rgb(0xF5F5F5))
            .rounded(px(6.0))
            .border(px(1.5))
            .border_color(rgb(0xFF9800))
            .child(
                div()
                    .child(format!("Undefined Node Type"))
                    .text_color(rgb(0x5F6368)),
            )
            .into_any()
    }
}
