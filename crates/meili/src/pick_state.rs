//! Dangling type picker: shared “are we picking a node type?” state between `FlowCanvas` plugins and
//! [`crate::shell::MeiliShell`]. gpui-component [`Select`](gpui_component::select::Select) must live in a view
//! with [`gpui::Context`], not in Ferrum `Plugin::render`.

use std::sync::Mutex;

use crate::plugins::pick_link_event::PickNodeTypeForPendingLink;

static MEILI_PENDING_NODE_PICK: Mutex<Option<PickNodeTypeForPendingLink>> = Mutex::new(None);

pub fn pending_peek() -> Option<PickNodeTypeForPendingLink> {
    MEILI_PENDING_NODE_PICK
        .lock()
        .ok()
        .and_then(|g| g.as_ref().copied())
}

pub fn pending_set(p: Option<PickNodeTypeForPendingLink>) {
    if let Ok(mut g) = MEILI_PENDING_NODE_PICK.lock() {
        *g = p;
    }
}

pub fn pending_take() -> Option<PickNodeTypeForPendingLink> {
    MEILI_PENDING_NODE_PICK.lock().ok().and_then(|mut g| g.take())
}
