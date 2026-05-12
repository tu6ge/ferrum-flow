//! Automatic graph layout.
//!
//! - [`strategy`] — [`LayoutStrategy`], [`LayoutOptions`], [`LayoutOutput`], [`LayoutError`].
//! - [`LayeredDagLayout`] — first built-in: longest-path layers, cycle → grid fallback.
//! - [`AutoLayoutPlugin`] — optional single strategy + shortcut; see that type’s docs.

mod auto_layout;
pub mod layered_dag;
mod strategy;

pub use auto_layout::AutoLayoutPlugin;
pub use layered_dag::LayeredDagLayout;
pub use strategy::{
    LayoutDirection, LayoutError, LayoutOptions, LayoutOutput, LayoutStrategy, NodePositionDelta,
};
