//! 悬垂选节点：在 `FlowCanvas` 插件与外层 [`crate::shell::MeiliShell`] 之间共享「当前是否在选类型」状态。
//! `gpui-component` 的 [`Select`](gpui_component::select::Select) 只能画在带 [`gpui::Context`] 的视图里，不能画在 Ferrum 的 `Plugin::render` 中。

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
