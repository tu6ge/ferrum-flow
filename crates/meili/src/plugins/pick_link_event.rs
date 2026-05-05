//! Custom events for dangling edges: after the user clicks the blue endpoint, the host picks a node type and
//! completes the link via [`crate::commit_commands::NodeTypeSelectConfirmCommand`].
//!
//! **Relationship to core**: [`PickNodeTypeForPendingLink`] is emitted from
//! [`super::meili_port_interaction::MeiliPortInteractionPlugin`] as a `FlowEvent::custom` downcast; completing the
//! link uses [`ferrum_flow::FlowCanvas::dispatch_command`] from [`crate::shell::MeiliShell`], not another custom event.

use ferrum_flow::PortId;
use gpui::{Pixels, Point};

#[derive(Clone, Copy)]
pub struct PickNodeTypeForPendingLink {
    pub source_port: PortId,
    pub end_world: Point<Pixels>,
}

