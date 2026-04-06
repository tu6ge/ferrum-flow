//! Applies [`crate::theme::apply_flow_chrome`] during [`Plugin::setup`](ferrum_flow::Plugin::setup).
//!
//! Register this plugin **before** other Meili / core plugins so [`ferrum_flow::FlowTheme`] is
//! populated before first render.

use ferrum_flow::{InitPluginContext, Plugin};

pub struct MeiliThemePlugin;

impl MeiliThemePlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for MeiliThemePlugin {
    fn name(&self) -> &'static str {
        "meili_theme"
    }

    fn setup(&mut self, ctx: &mut InitPluginContext) {
        crate::theme::apply_flow_chrome(ctx.theme);
    }
}
