//! Canvas “Add node” context menu: handles [`AddNodeConfirm`](super::pick_link_event::AddNodeConfirm) from the
//! Shell and creates a node at the right-click world position with the chosen type.

use crate::add_node_dialog;
use crate::plugins::node_kind_preset::preset_for_digit;
use crate::plugins::pick_link_event::AddNodeConfirm;
use ferrum_flow::{
    CreateNode, CreatePort, EventResult, FlowEvent, InputEvent, Plugin, PluginContext, RenderLayer,
};

pub struct MeiliAddNodePlugin;

impl MeiliAddNodePlugin {
    pub fn new() -> Self {
        Self
    }

    fn commit(ctx: &mut PluginContext, label: &str, world_x: f32, world_y: f32, kind_digit: u8) {
        let Some(preset) = preset_for_digit(kind_digit).or_else(|| preset_for_digit(7)) else {
            return;
        };
        let (node_type, w, h, data) = preset.describe_with_title(label);
        let x = world_x - w * 0.5;
        let y = world_y - h * 0.5;
        let builder = ctx
            .create_node(node_type)
            .position(x, y)
            .size(w, h)
            .data(data)
            .execute_type(node_type);
        let builder = preset.apply_standalone_ports(builder);
        let (new_node, new_ports) = builder.only_build();
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
                Self::commit(ctx, s, c.world_x, c.world_y, c.kind_digit);
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
