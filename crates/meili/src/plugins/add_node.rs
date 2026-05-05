//! Canvas “Add node” context menu: Esc while the add-node dialog is open closes it (graph changes use
//! [`crate::commit_commands::AddNodeConfirmCommand`] from the shell).

use crate::add_node_dialog;
use ferrum_flow::{EventResult, FlowEvent, InputEvent, Plugin, PluginContext, RenderLayer};

pub struct MeiliAddNodePlugin;

impl MeiliAddNodePlugin {
    pub fn new() -> Self {
        Self
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
