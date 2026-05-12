//! Automatic graph layout (**scaffold**).
//!
//! Layered DAG, force-directed, and tree passes will live here. [`compute`] is the single entry
//! point the [`super::AutoLayoutPlugin`](crate::plugins::AutoLayoutPlugin) calls; it currently
//! returns [`AutoLayoutComputeResult::Pending`] when the graph has nodes so the UI can show a
//! hint until real coordinates are produced.

use crate::Graph;

mod auto_layout;

pub use auto_layout::AutoLayoutPlugin;

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
