//! Subtle top bar with product title — sits on the overlay layer.

use ferrum_flow::{
    EventResult, FlowEvent, InitPluginContext, Plugin, PluginContext, RenderContext, RenderLayer,
};
use gpui::{Element as _, ParentElement as _, Styled, div, px, rgb, rgba};

use crate::theme::{HUD_BAR_BG_RGBA, HUD_BAR_BORDER_RGBA, TEXT_MUTED, TEXT_PRIMARY};

pub struct AgentHudPlugin;

impl AgentHudPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for AgentHudPlugin {
    fn name(&self) -> &'static str {
        "meili_agent_hud"
    }

    fn setup(&mut self, _ctx: &mut InitPluginContext) {}

    fn on_event(
        &mut self,
        _event: &FlowEvent,
        _ctx: &mut PluginContext,
    ) -> EventResult {
        EventResult::Continue
    }

    fn priority(&self) -> i32 {
        5
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let bounds = ctx.window.bounds();
        let w = bounds.size.width;

        Some(
            div()
                .absolute()
                .top(px(0.0))
                .left(px(0.0))
                .w(w)
                .h(px(40.0))
                .flex()
                .items_center()
                .px(px(16.0))
                .border_b(px(1.0))
                .border_color(rgba(HUD_BAR_BORDER_RGBA))
                .bg(rgba(HUD_BAR_BG_RGBA))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .child("Meili")
                                .text_color(rgb(TEXT_PRIMARY))
                                .text_size(px(14.0)),
                        )
                        .child(
                            div()
                                .child("Agent workflow studio")
                                .text_color(rgb(TEXT_MUTED))
                                .text_size(px(11.0)),
                        ),
                )
                .into_any(),
        )
    }
}
