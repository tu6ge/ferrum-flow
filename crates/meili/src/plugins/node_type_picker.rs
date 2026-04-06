//! # 悬垂连线后的节点类型选择（Meili）
//!
//! ## 流程说明
//!
//! 1. 从某个端口拖出一条连线，在**空白画布**上松开鼠标 → 进入「悬垂连线」状态（灰线 + 蓝色端点）。
//! 2. 点击**蓝色端点** → [`MeiliPortInteractionPlugin`](super::meili_port_interaction::MeiliPortInteractionPlugin)
//!    发出 [`PickNodeTypeForPendingLink`](super::pick_link_event::PickNodeTypeForPendingLink)；
//!    本插件把该请求写入 [`crate::pick_state`]，并 `notify` 画布。
//! 3. 外层 [`crate::shell::MeiliShell`] 发现 `pick_state` 有值时，渲染 **gpui-component** 的
//!    [`Select`](gpui_component::select::Select)；用户选定后 Shell 调用 [`ferrum_flow::FlowCanvas::handle_event`]
//!    投递 [`NodeTypeSelectConfirm`](super::pick_link_event::NodeTypeSelectConfirm)，由本插件 `commit_choice`。
//!
//! ## 为何不用 `Select` 画在插件里
//!
//! `Select` 依赖 `gpui::Context` 与组件内部实体；Ferrum 的 `Plugin::render` 只有 [`ferrum_flow::RenderContext`]，
//! 因此下拉控件必须放在窗口子视图（本仓库的 `MeiliShell`）中。
//!
//! ## 操作方式
//!
//! - 使用底部 **下拉框**（gpui-component Select）选择类型；支持组件自带搜索/键盘导航。
//! - **Esc** 仍可取消（在画布焦点上由本插件处理）。

use crate::pick_state;
use crate::plugins::node_kind_preset::{NodeKindPreset, preset_for_digit};
use crate::plugins::pick_link_event::{NodeTypeSelectConfirm, PickNodeTypeForPendingLink};
use ferrum_flow::{
    CreateEdge, CreateNode, CreatePort, EventResult, FlowEvent, InputEvent, NodeBuilder, Plugin,
    PluginContext, PortKind, PortPosition, RenderContext, RenderLayer, edge_bezier, filled_disc_path,
    port_screen_position,
};
use gpui::{Element as _, ParentElement as _, Styled, canvas, div, px, rgb};

pub struct NodeTypePickerPlugin;

impl NodeTypePickerPlugin {
    pub fn new() -> Self {
        Self
    }

    fn facing_position(p: PortPosition) -> PortPosition {
        match p {
            PortPosition::Left => PortPosition::Right,
            PortPosition::Right => PortPosition::Left,
            PortPosition::Top => PortPosition::Bottom,
            PortPosition::Bottom => PortPosition::Top,
        }
    }

    fn with_opposite_port(kind: PortKind, preset: NodeKindPreset, b: NodeBuilder) -> NodeBuilder {
        match kind {
            PortKind::Output => match preset {
                NodeKindPreset::Tool => b.input_at(PortPosition::Top),
                _ => b.input(),
            },
            PortKind::Input => match preset {
                NodeKindPreset::Tool => b.output_at(PortPosition::Bottom),
                _ => b.output(),
            },
        }
    }

    fn commit_choice(ctx: &mut PluginContext, choice: NodeKindPreset) {
        let Some(p) = pick_state::pending_take() else {
            return;
        };
        let Some(source) = ctx.graph.ports.get(&p.source_port).cloned() else {
            return;
        };

        let x: f32 = p.end_world.x.into();
        let y: f32 = p.end_world.y.into();

        let (node_type, w, h, data) = choice.describe();

        let mut builder = ctx.create_node(node_type);
        builder = builder
            .position(x, y)
            .size(w, h)
            .data(data)
            .execute_type(node_type);
        builder = Self::with_opposite_port(source.kind, choice, builder);

        let (new_node, new_ports) = builder.only_build(ctx.graph);

        let edge = match source.kind {
            PortKind::Output => {
                let Some(in_port) = new_node.inputs.first().copied() else {
                    return;
                };
                ctx.new_edge().source(p.source_port).target(in_port)
            }
            PortKind::Input => {
                let Some(out_port) = new_node.outputs.first().copied() else {
                    return;
                };
                ctx.new_edge().source(out_port).target(p.source_port)
            }
        };

        ctx.execute_command(CreateNode::new(new_node));
        for port in new_ports {
            ctx.execute_command(CreatePort::new(port));
        }
        ctx.execute_command(CreateEdge::new(edge));
        ctx.notify();
    }
}

impl Plugin for NodeTypePickerPlugin {
    fn name(&self) -> &'static str {
        "meili_node_type_picker"
    }

    fn setup(&mut self, _ctx: &mut ferrum_flow::InitPluginContext) {}

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if let Some(p) = event.as_custom::<PickNodeTypeForPendingLink>() {
            pick_state::pending_set(Some(*p));
            ctx.notify();
            return EventResult::Stop;
        }

        if let Some(c) = event.as_custom::<NodeTypeSelectConfirm>() {
            if let Some(preset) = preset_for_digit(c.digit) {
                Self::commit_choice(ctx, preset);
            }
            return EventResult::Stop;
        }

        if pick_state::pending_peek().is_none() {
            return EventResult::Continue;
        }

        if let FlowEvent::Input(InputEvent::KeyDown(ev)) = event {
            if ev.keystroke.key == "escape" {
                pick_state::pending_set(None);
                ctx.notify();
                return EventResult::Stop;
            }
        }

        EventResult::Continue
    }

    fn priority(&self) -> i32 {
        130
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    /// 只画悬垂线 + 蓝点；类型选择 UI 由 [`crate::shell::MeiliShell`] 中的 `gpui-component` Select 负责。
    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let p = pick_state::pending_peek()?;
        let port = ctx.graph.ports.get(&p.source_port)?;
        let node = ctx.nodes().get(&port.node_id)?;
        let start = port_screen_position(node, p.source_port, ctx)?;
        let end = ctx.world_to_screen(p.end_world);
        let start_position = port.position;
        let target_position = Self::facing_position(start_position);
        let viewport = ctx.viewport.clone();
        let line_rgb = ctx.theme.port_preview_line;
        let dot_rgb = ctx.theme.port_preview_dot;

        let wire = canvas(
            move |_, _, _| (),
            move |_, _, win, _| {
                if let Ok(line) =
                    edge_bezier(start, start_position, target_position, end, &viewport)
                {
                    win.paint_path(line, rgb(line_rgb));
                }
                if let Ok(dot) = filled_disc_path(end, px(6.0)) {
                    win.paint_path(dot, rgb(dot_rgb));
                }
            },
        );

        Some(
            div()
                .absolute()
                .top(px(0.0))
                .left(px(0.0))
                .size_full()
                .child(wire)
                .into_any(),
        )
    }
}
