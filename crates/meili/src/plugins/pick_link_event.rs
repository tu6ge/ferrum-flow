//! 自定义事件：悬垂连线蓝点被点击后，请求宿主选择节点类型再落点连线。
//!
//! **与 core 的关系**：理想情况下该类型应由 `ferrum-flow` 定义并在 `PortInteractionPlugin` 里 `emit`
//!（与 `FlowEvent::custom` 的 downcast 类型一致）。在你确认可以改 core 之前，Meili 使用本模块的副本，
//! 并由 [`super::meili_port_interaction::MeiliPortInteractionPlugin`] 发出，避免与仓库内未升级的 core 冲突。

use ferrum_flow::PortId;
use gpui::{Pixels, Point, SharedString};

#[derive(Clone, Copy)]
pub struct PickNodeTypeForPendingLink {
    pub source_port: PortId,
    pub end_world: Point<Pixels>,
}

/// 由外层 [`crate::shell::MeiliShell`] 在用户从 `gpui-component` Select 选定一项后投递给 [`ferrum_flow::FlowCanvas::handle_event`]。
#[derive(Clone, Copy)]
pub struct NodeTypeSelectConfirm {
    pub digit: u8,
}

/// 由 Shell 在用户确认「添加节点」输入后投递；[`crate::plugins::MeiliAddNodePlugin`] 负责落点与写回图。
#[derive(Clone)]
pub struct AddNodeConfirm {
    pub label: SharedString,
    pub world_x: f32,
    pub world_y: f32,
    /// Same encoding as [`NodeTypeSelectConfirm::digit`] / bottom-bar type picker (1–7).
    pub kind_digit: u8,
}
