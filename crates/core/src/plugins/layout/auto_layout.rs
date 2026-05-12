//! Keyboard entry point for automatic graph layout.
//!
//! Layout geometry is delegated to [`crate::plugins::layout`]. Until algorithms land, triggering
//! layout shows an optional info toast (when [`crate::ToastPlugin`] is registered).
//!
//! Shortcut: **primary modifier + Shift + G** (e.g. ⌘⇧G / Ctrl⇧G), chosen to avoid overlap with
//! [`crate::AlignPlugin`](crate::plugins::AlignPlugin) (⌘⇧L/R/T/B/H/V).

use crate::{
    ToastMessage,
    plugin::{FlowEvent, Plugin, PluginContext, primary_platform_modifier},
};

use super::{AutoLayoutComputeResult, compute};

/// Triggers automatic layout; algorithms live in [`crate::plugins::layout`].
pub struct AutoLayoutPlugin;

impl AutoLayoutPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AutoLayoutPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AutoLayoutPlugin {
    fn name(&self) -> &'static str {
        "auto_layout"
    }

    fn priority(&self) -> i32 {
        89
    }

    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut PluginContext,
    ) -> crate::plugin::EventResult {
        if let FlowEvent::Input(crate::plugin::InputEvent::KeyDown(ev)) = event {
            if primary_platform_modifier(ev)
                && ev.keystroke.modifiers.shift
                && ev.keystroke.key == "g"
            {
                match compute(ctx.graph) {
                    AutoLayoutComputeResult::NoNodes => {}
                    AutoLayoutComputeResult::Pending => {
                        ctx.emit(FlowEvent::custom(ToastMessage::info(
                            "Auto-layout: algorithms not implemented yet (see plugins::layout).",
                        )));
                    }
                }
                return crate::plugin::EventResult::Stop;
            }
        }
        crate::plugin::EventResult::Continue
    }
}
