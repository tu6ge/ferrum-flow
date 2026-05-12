//! Automatic graph layout.
//!
//! - [`strategy`] — [`LayoutStrategy`], [`LayoutOptions`], [`LayoutOutput`], [`LayoutError`].
//! - [`AutoLayoutPlugin`] — optional single strategy + shortcut; see that type’s docs.

mod auto_layout;
mod strategy;

pub use auto_layout::AutoLayoutPlugin;
pub use strategy::{
    LayoutDirection, LayoutError, LayoutOptions, LayoutOutput, LayoutStrategy, NodePositionDelta,
};
