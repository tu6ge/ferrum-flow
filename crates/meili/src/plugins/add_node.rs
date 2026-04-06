//! 画布菜单「添加节点」：接收 Shell 投递的 [`AddNodeConfirm`](super::pick_link_event::AddNodeConfirm)，在右键世界坐标处创建通用步骤节点。

use crate::add_node_dialog;
use crate::plugins::pick_link_event::AddNodeConfirm;
use ferrum_flow::{
    CreateNode, CreatePort, EventResult, FlowEvent, InputEvent, Plugin, PluginContext, RenderLayer,
};
use serde_json::json;

pub struct MeiliAddNodePlugin;

impl MeiliAddNodePlugin {
    pub fn new() -> Self {
        Self
    }

    fn commit(ctx: &mut PluginContext, label: &str, world_x: f32, world_y: f32) {
        // Center the default 200×96 card on the right-click world point.
        let x = world_x - 100.0;
        let y = world_y - 48.0;
        let data = json!({
            "title": label,
            "subtitle": "步骤"
        });
        let builder = ctx
            .create_node("")
            .position(x, y)
            .size(200.0, 96.0)
            .data(data)
            .execute_type("")
            .input()
            .output();
        let (new_node, new_ports) = builder.only_build(ctx.graph);
        ctx.execute_command(CreateNode::new(new_node));
        for port in new_ports {
            ctx.execute_command(CreatePort::new(port));
        }
        ctx.notify();
    }
}

impl Plugin for MeiliAddNodePlugin {
    fn name(&self) -> &'static str {
        "meili_add_node"
    }

    fn priority(&self) -> i32 {
        131
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if let Some(c) = event.as_custom::<AddNodeConfirm>() {
            let s = c.label.trim();
            if !s.is_empty() {
                Self::commit(ctx, s, c.world_x, c.world_y);
            }
            return EventResult::Stop;
        }

        if add_node_dialog::is_open() {
            if let FlowEvent::Input(InputEvent::KeyDown(ev)) = event {
                if ev.keystroke.key == "escape" {
                    add_node_dialog::close();
                    ctx.notify();
                    return EventResult::Stop;
                }
            }
        }

        EventResult::Continue
    }
}
