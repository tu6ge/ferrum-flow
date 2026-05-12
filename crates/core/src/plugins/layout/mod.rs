//! Automatic graph layout.
//!
//! - [`strategy`] — [`LayoutStrategy`], [`LayoutPhase`], [`PositionHint`], [`LayoutOptions`], …
//! - [`layered_dag`] — [`LayeredDagLayout`] (longest-path layers; cycles → grid fallback).
//! - [`force_directed`] — [`ForceDirectedLayout`] (FR-style; cycles OK).
//! - [`layered_then_force`] — [`LayeredThenForceLayout`] (layered warm start → force).
//! - [`AutoLayoutPlugin`] — optional single strategy + shortcut; see that type’s docs.

mod auto_layout;
pub mod force_directed;
pub mod layered_dag;
mod layered_then_force;
mod strategy;

pub use auto_layout::AutoLayoutPlugin;
pub use force_directed::ForceDirectedLayout;
pub use layered_dag::LayeredDagLayout;
pub use layered_then_force::LayeredThenForceLayout;
pub use strategy::{
    LayoutDirection, LayoutError, LayoutOptions, LayoutOutput, LayoutPhase, LayoutStrategy,
    NodePositionDelta, PositionHint,
};
