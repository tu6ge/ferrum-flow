//! Automatic graph layout (**scaffold**).
//!
//! - [`strategy`] — [`LayoutStrategy`], [`LayoutOptions`], [`LayoutOutput`], [`LayoutError`].
//! - [`compute`] — temporary stub used by [`AutoLayoutPlugin`](auto_layout::AutoLayoutPlugin);
//!   new algorithms should implement [`LayoutStrategy`] and be invoked from the plugin instead.

use crate::Graph;

mod auto_layout;
mod strategy;

pub use auto_layout::AutoLayoutPlugin;
pub use strategy::{
    LayoutDirection, LayoutError, LayoutOptions, LayoutOutput, LayoutStrategy, NodePositionDelta,
};

/// Outcome of a layout pass over the current graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoLayoutComputeResult {
    /// Nothing to arrange.
    NoNodes,
    /// Algorithms not implemented yet; no positions are applied.
    Pending,
    // Future: `Ready` with from/to snapshots for `DragNodesCommand`, etc.
}

/// Computes new node positions. **Stub:** returns [`AutoLayoutComputeResult::Pending`] if there
/// is at least one node; otherwise [`AutoLayoutComputeResult::NoNodes`].
pub fn compute(graph: &Graph) -> AutoLayoutComputeResult {
    if graph.nodes().is_empty() {
        AutoLayoutComputeResult::NoNodes
    } else {
        AutoLayoutComputeResult::Pending
    }
}
