use std::collections::{HashMap, HashSet};

use gpui::{Bounds, Pixels, Point};

use crate::{NodeId, plugin::Plugin};

pub struct SelectionPlugin {
    selected_nodes: HashSet<NodeId>,
    //selected_edges: HashSet<EdgeId>,
    box_selection: Option<BoxSelection>,
}

impl SelectionPlugin {
    pub fn new() -> Self {
        Self {
            selected_nodes: HashSet::new(),
            box_selection: None,
        }
    }
}

impl Plugin for SelectionPlugin {
    fn name(&self) -> &'static str {
        "selection"
    }
    fn setup(&mut self, ctx: &mut crate::plugin::PluginContext) {}
    fn on_event(
        &mut self,
        event: &crate::plugin::FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) {
    }
}

#[derive(Debug, Clone)]
pub struct BoxSelection {
    pub(super) start_mouse: Point<Pixels>,
    pub(super) bounds: Bounds<Pixels>,
    pub(super) nodes: HashMap<NodeId, Point<Pixels>>,
}

#[derive(Debug, Clone)]
pub struct PendingBoxSelect {
    pub(super) start: Point<Pixels>,
}
