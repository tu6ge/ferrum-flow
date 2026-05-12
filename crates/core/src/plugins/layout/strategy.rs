//! Pluggable graph layout: [`LayoutStrategy`] and shared input/output types.
//!
//! Implementations live in sibling modules (e.g. layered DAG, force). The
//! [`crate::plugins::layout::AutoLayoutPlugin`](super::AutoLayoutPlugin) (or a host shell) picks a
//! strategy and applies [`LayoutOutput::Delta`] via [`crate::plugins::node::DragNodesCommand`].

use std::fmt;

use gpui::{Pixels, Point};

use crate::{Graph, NodeId};

/// Tunable knobs for layout passes. Marked [`non_exhaustive`] so new fields can be added without
/// breaking downstream struct literals.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct LayoutOptions {
    /// World-space gap between adjacent layers (e.g. left-to-right DAG).
    pub layer_spacing: f32,
    /// World-space gap between nodes in the same layer.
    pub sibling_spacing: f32,
    /// Primary flow direction for layered layouts.
    pub direction: LayoutDirection,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            layer_spacing: 120.0,
            sibling_spacing: 48.0,
            direction: LayoutDirection::LeftToRight,
        }
    }
}

/// Primary axis along which “layers” advance for layered algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutDirection {
    #[default]
    LeftToRight,
    TopToBottom,
}

/// Before / after node positions for one layout pass (same length, paired by index).
#[derive(Debug, Clone)]
pub struct NodePositionDelta {
    from: Vec<(NodeId, Point<Pixels>)>,
    to: Vec<(NodeId, Point<Pixels>)>,
}

impl NodePositionDelta {
    pub fn new(from: Vec<(NodeId, Point<Pixels>)>, to: Vec<(NodeId, Point<Pixels>)>) -> Self {
        Self { from, to }
    }

    /// Returns `true` if any node position changed (using raw pixel comparison).
    pub fn has_changes(&self) -> bool {
        self.from.len() == self.to.len()
            && self
                .from
                .iter()
                .zip(self.to.iter())
                .any(|((_, a), (_, b))| a != b)
    }
}

/// Result of a successful layout computation (no fatal error).
#[derive(Debug, Clone)]
pub enum LayoutOutput {
    /// No moves to apply (empty graph, filtered set, or already satisfied).
    Unchanged,
    /// Apply with [`crate::plugins::node::DragNodesCommand::from_positions`].
    Delta(NodePositionDelta),
}

/// Recoverable layout failure (graph shape, missing data, unsupported case).
#[derive(Debug, Clone)]
pub enum LayoutError {
    /// Graph has no nodes to lay out.
    EmptyGraph,
    /// Directed cycle prevents a pure layered ordering (caller may choose another strategy).
    CycleInGraph,
    /// User-visible or loggable detail.
    Message(String),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGraph => write!(f, "empty graph"),
            Self::CycleInGraph => write!(f, "graph contains a directed cycle"),
            Self::Message(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for LayoutError {}

/// One layout algorithm (layered DAG, force, grid, …). Register multiple behind
/// [`super::AutoLayoutPlugin`](super::AutoLayoutPlugin) or a small `Vec`/`HashMap` by [`Self::id`].
///
/// Object-safe: use `&Graph` and owned output; keep impls [`Send`] + [`Sync`] so the canvas can
/// hold `Arc<dyn LayoutStrategy>` across threads if needed.
pub trait LayoutStrategy: Send + Sync {
    /// Stable id for menus, config, and logging (e.g. `"layered_dag"`).
    fn id(&self) -> &'static str;

    /// Short human-readable label (optional UI).
    fn label(&self) -> &'static str {
        self.id()
    }

    /// Compute new positions. Must not mutate `graph`; callers apply [`LayoutOutput::Delta`].
    fn compute(&self, graph: &Graph, options: &LayoutOptions) -> Result<LayoutOutput, LayoutError>;
}
