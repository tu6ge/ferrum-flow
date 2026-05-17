//! GPUI-oriented **host-agnostic** plugin surface: lifecycle, input/events, and optional views.
//!
//! This crate is intentionally **decoupled** from `ferrum-flow` (no graph / canvas types). A host application implements concrete `InitContext`, `Event`, `ExecContext`, and
//! `ViewContext` context types and passes them as type parameters to [`RawPlugin`].
//!
//! ## Roadmap (incremental)
//!
//! 1. **Current**: [`RawPlugin`] + [`EventPropagation`].
//! 2. Next: typed plugin id, registration / ordered dispatch helpers.
//! 3. Later: optional “ECS” helpers (component buckets, system ordering) without mandating one
//!    storage strategy.

use gpui::AnyElement;

/// Whether the host should keep dispatching after this plugin handles an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EventPropagation {
    #[default]
    Continue,
    Stop,
}

/// Extension point for a GPUI host: **generic** init / event / exec / render contexts and render layer.
pub trait CanvasPlugin<InitContext, Event, ExecContext, ViewContext, RenderLayer> {
    fn name(&self) -> &'static str;

    fn setup(&mut self, _ctx: &mut InitContext) {}

    fn on_event(&mut self, _event: &Event, _ctx: &mut ExecContext) -> EventPropagation {
        EventPropagation::Continue
    }

    fn render(&mut self, _ctx: &mut ViewContext) -> Option<AnyElement> {
        None
    }

    fn priority(&self) -> i32 {
        0
    }

    fn render_layer(&self) -> RenderLayer;
}
