//! Automatic graph layout.
//!
//! - [`strategy`] — [`LayoutStrategy`], [`LayoutOptions`], [`LayoutOutput`], [`LayoutError`].
//! - [`layered_dag`] — [`LayeredDagLayout`] (longest-path layers; cycles → grid fallback).
//! - [`force_directed`] — [`ForceDirectedLayout`] (FR-style; cycles OK).
//! - [`AutoLayoutPlugin`] — optional single strategy + shortcut; see that type’s docs.

mod auto_layout;
pub mod force_directed;
pub mod layered_dag;
mod strategy;

pub use auto_layout::AutoLayoutPlugin;
pub use force_directed::ForceDirectedLayout;
pub use layered_dag::LayeredDagLayout;
pub use strategy::{
    LayoutDirection, LayoutError, LayoutOptions, LayoutOutput, LayoutStrategy, NodePositionDelta,
};
