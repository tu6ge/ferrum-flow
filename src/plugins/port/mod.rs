use crate::{Port, plugin::Plugin};

mod interaction;
mod utils;
pub use utils::*;

use gpui::{Element, ParentElement, Styled as _, div, px, rgb};
pub use interaction::PortInteractionPlugin;

mod command;

pub struct PortPlugin;

impl PortPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for PortPlugin {
    fn name(&self) -> &'static str {
        "port"
    }
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn priority(&self) -> i32 {
        70
    }
    fn render_layer(&self) -> crate::plugin::RenderLayer {
        crate::plugin::RenderLayer::Ports
    }
    fn render(&mut self, ctx: &mut crate::RenderContext) -> Option<gpui::AnyElement> {
        let list: Vec<_> = ctx
            .graph
            .ports
            .iter()
            .filter_map(|(_, Port { id, .. })| {
                let position = port_screen_position(*id, &ctx)?;

                Some(
                    div()
                        .absolute()
                        .left(position.x - px(6.0 * ctx.viewport.zoom))
                        .top(position.y - px(6.0 * ctx.viewport.zoom))
                        .w(px(12.0 * ctx.viewport.zoom))
                        .h(px(12.0 * ctx.viewport.zoom))
                        .rounded_full()
                        .bg(rgb(0x1A192B)),
                )
            })
            .collect();

        Some(div().children(list).into_any())
    }
}
