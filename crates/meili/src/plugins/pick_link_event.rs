//! Custom events for dangling edges: after the user clicks the blue endpoint, the host picks a node type and
//! completes the link.
//!
//! **Relationship to core**: ideally these types would live in `ferrum-flow` and be emitted from
//! `PortInteractionPlugin` (matching `FlowEvent::custom` downcasts). Until core is changed, Meili keeps copies
//! here and emits them from [`super::meili_port_interaction::MeiliPortInteractionPlugin`] to avoid version skew.

use ferrum_flow::PortId;
use gpui::{Pixels, Point, SharedString};

#[derive(Clone, Copy)]
pub struct PickNodeTypeForPendingLink {
    pub source_port: PortId,
    pub end_world: Point<Pixels>,
}

/// Dispatched by [`crate::shell::MeiliShell`] when the user confirms a row in the gpui-component `Select`.
#[derive(Clone, Copy)]
pub struct NodeTypeSelectConfirm {
    pub digit: u8,
}

/// Dispatched by the Shell after the user confirms the "Add node" dialog; [`crate::plugins::MeiliAddNodePlugin`]
/// creates the node and updates the graph.
#[derive(Clone)]
pub struct AddNodeConfirm {
    pub label: SharedString,
    pub world_x: f32,
    pub world_y: f32,
    /// Same encoding as [`NodeTypeSelectConfirm::digit`] / bottom-bar type picker (1–7).
    pub kind_digit: u8,
}
