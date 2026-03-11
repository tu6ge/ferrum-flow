use std::collections::HashMap;

use gpui::{Bounds, Pixels, Point};

use crate::NodeId;

#[derive(Debug, Clone)]
pub enum DragState {
    None,
    NodeDrag(NodeDrag),
    PendingBoxSelect(PendingBoxSelect),
    BoxSelect(BoxSelectDrag),
    BoxMove(BoxMoveDrag),
    Pan(Panning),
    PendingNode(PendingNode),
    EdgeDrag(Connecting),
}

#[derive(Debug, Clone)]
pub struct NodeDrag {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) start_positions: Vec<(NodeId, Point<Pixels>)>,
}

#[derive(Debug, Clone)]
pub struct BoxMoveDrag {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) start_bounds: Bounds<Pixels>,
    pub(super) nodes: Vec<(NodeId, Point<Pixels>)>,
}

#[derive(Debug, Clone)]
pub struct BoxSelection {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) bounds: Bounds<Pixels>,
    pub(super) nodes: HashMap<NodeId, Point<Pixels>>,
}

#[derive(Debug, Clone)]
pub struct PendingNode {
    pub(super) node_id: NodeId,
    pub(super) start_mouse: Point<Pixels>,
    pub(super) shift: bool,
}

#[derive(Debug, Clone)]
pub struct Connecting {
    pub(super) node_id: NodeId,
    pub(super) port_id: String,
    pub(super) mouse: Point<Pixels>,
}

#[derive(Debug, Clone)]
pub struct Panning {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) start_offset: Point<Pixels>,
}

#[derive(Debug, Clone)]
pub struct PendingBoxSelect {
    pub(super) start: Point<Pixels>,
}

#[derive(Debug, Clone)]
pub struct BoxSelectDrag {
    pub(super) start: Point<Pixels>,
    pub(super) end: Point<Pixels>,
}
