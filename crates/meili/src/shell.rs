//! 窗口根视图：在 [`ferrum_flow::FlowCanvas`] 之上叠一层 UI。
//!
//! [`gpui_component::select::Select`] 需要在带 [`gpui::Context`] 的 `Render` 里构建，因此放在本模块，而不是
//! `NodeTypePickerPlugin::render`。用户从下拉框选定后，通过 [`ferrum_flow::FlowCanvas::handle_event`] 投递
//! [`NodeTypeSelectConfirm`](crate::plugins::pick_link_event::NodeTypeSelectConfirm)，由插件完成建点与连线。

use ferrum_flow::FlowCanvas;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, px,
};
use gpui_component::select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState};
use gpui_component::{Root, Sizable as _};

use crate::pick_state;
use crate::plugins::pick_link_event::NodeTypeSelectConfirm;

#[derive(Clone)]
struct NodePickItem {
    title: SharedString,
    value: u8,
}

impl SelectItem for NodePickItem {
    type Value = u8;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &u8 {
        &self.value
    }
}

fn node_pick_items() -> SearchableVec<NodePickItem> {
    SearchableVec::new([
        NodePickItem {
            title: "Agent — 编排 · 工具 · 记忆".into(),
            value: 1,
        },
        NodePickItem {
            title: "LLM — 模型推理".into(),
            value: 2,
        },
        NodePickItem {
            title: "Tool — 搜索 / 代码 / RAG".into(),
            value: 3,
        },
        NodePickItem {
            title: "Router — 分支 / 重试".into(),
            value: 4,
        },
        NodePickItem {
            title: "起点 — io_start".into(),
            value: 5,
        },
        NodePickItem {
            title: "终点 — io_end".into(),
            value: 6,
        },
        NodePickItem {
            title: "步骤 — 通用".into(),
            value: 7,
        },
    ])
}

pub struct MeiliShell {
    pub canvas: Entity<FlowCanvas>,
    node_select: Entity<SelectState<SearchableVec<NodePickItem>>>,
    _canvas_obs: Subscription,
    _select_sub: Subscription,
}

impl MeiliShell {
    pub fn new(canvas: Entity<FlowCanvas>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let node_select = cx.new(|cx| SelectState::new(node_pick_items(), None, window, cx));

        let canvas_obs = cx.observe(&canvas, |_shell, _, cx| {
            cx.notify();
        });

        let canvas_for_confirm = canvas.clone();
        let select_sub = cx.subscribe(
            &node_select,
            move |_shell, _, event: &SelectEvent<SearchableVec<NodePickItem>>, cx| {
                if let SelectEvent::Confirm(Some(digit)) = event {
                    canvas_for_confirm.update(cx, |flow, cx| {
                        flow.handle_event(
                            ferrum_flow::FlowEvent::custom(NodeTypeSelectConfirm { digit: *digit }),
                            cx,
                        );
                    });
                }
            },
        );

        Self {
            canvas,
            node_select,
            _canvas_obs: canvas_obs,
            _select_sub: select_sub,
        }
    }
}

impl Render for MeiliShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let show_bar = pick_state::pending_peek().is_some();

        div()
            .size_full()
            .relative()
            .child(self.canvas.clone())
            .when(show_bar, |this| {
                this.child(
                    div()
                        .absolute()
                        .bottom(px(24.0))
                        .left(px(0.0))
                        .right(px(0.0))
                        .flex()
                        .justify_center()
                        .child(
                            div().w(px(380.0)).child(
                                Select::new(&self.node_select)
                                    .placeholder("选择节点类型 / Choose node type")
                                    .menu_width(px(380.0))
                                    .small(),
                            ),
                        ),
                )
            })
    }
}

/// 由 [`crate::main`] 构造：`Root` 作为窗口第一层（`gpui-component` 要求），内层为 [`MeiliShell`]。
pub fn window_root(
    canvas: Entity<FlowCanvas>,
    window: &mut Window,
    cx: &mut Context<Root>,
) -> Root {
    let shell = cx.new(|ctx| MeiliShell::new(canvas, window, ctx));
    Root::new(shell, window, cx)
}
