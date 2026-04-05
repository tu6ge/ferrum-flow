//! Dark canvas with a soft dot grid — replaces the default light [`BackgroundPlugin`](ferrum_flow::BackgroundPlugin).

use ferrum_flow::{FlowEvent, InitPluginContext, Plugin, PluginContext, RenderContext, RenderLayer};
use gpui::{Element, ParentElement, Styled, div, px, rgb};

use crate::theme::{CANVAS_BG, GRID_DOT};

pub struct AgentBackgroundPlugin;

impl AgentBackgroundPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for AgentBackgroundPlugin {
    fn name(&self) -> &'static str {
        "meili_agent_background"
    }

    fn setup(&mut self, _ctx: &mut InitPluginContext) {}

    fn on_event(
        &mut self,
        _event: &FlowEvent,
        _ctx: &mut PluginContext,
    ) -> ferrum_flow::EventResult {
        ferrum_flow::EventResult::Continue
    }

    fn priority(&self) -> i32 {
        0
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Background
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let base_grid = 48.0_f32;
        let zoom = ctx.viewport.zoom;
        let grid = base_grid * zoom;

        let offset = ctx.viewport.offset;
        let start_x = f32::from(offset.x).rem_euclid(grid);
        let start_y = f32::from(offset.y).rem_euclid(grid);

        let bounds = ctx.window.bounds();
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);

        let mut dots = Vec::new();
        let mut x = start_x;
        while x < width {
            let mut y = start_y;
            while y < height {
                dots.push(
                    div()
                        .absolute()
                        .left(px(x))
                        .top(px(y))
                        .w(px(1.5))
                        .h(px(1.5))
                        .rounded_full()
                        .bg(rgb(GRID_DOT)),
                );
                y += grid;
            }
            x += grid;
        }

        Some(
            div()
                .absolute()
                .w(px(width))
                .h(px(height))
                .bg(rgb(CANVAS_BG))
                .children(dots)
                .into_any(),
        )
    }
}
