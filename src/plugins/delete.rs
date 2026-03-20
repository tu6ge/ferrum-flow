use crate::{
    Edge, Node,
    canvas::Command,
    plugin::{FlowEvent, Plugin},
};

pub struct DeletePlugin;

impl DeletePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for DeletePlugin {
    fn name(&self) -> &'static str {
        "delete"
    }
    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}
    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut crate::plugin::PluginContext,
    ) -> crate::plugin::EventResult {
        if let FlowEvent::Input(crate::plugin::InputEvent::KeyDown(ev)) = event {
            if ev.keystroke.key == "delete" || ev.keystroke.key == "backspace" {
                ctx.execute_command(DeleteCommand::new(&ctx));
                return crate::plugin::EventResult::Stop;
            }
        }
        crate::plugin::EventResult::Continue
    }
}

struct DeleteCommand {
    selected_edge: Vec<Edge>,
    selected_node: Vec<Node>,
}

impl DeleteCommand {
    fn new(ctx: &crate::plugin::PluginContext) -> Self {
        Self {
            selected_edge: ctx
                .graph
                .selected_edge
                .iter()
                .filter_map(|id| ctx.graph.edges.get(id).cloned())
                .collect(),
            selected_node: ctx
                .graph
                .selected_node
                .iter()
                .filter_map(|id| ctx.get_node(id).cloned())
                .collect(),
        }
    }
}

impl Command for DeleteCommand {
    fn name(&self) -> &'static str {
        "delete"
    }
    fn execute(&mut self, ctx: &mut crate::canvas::CommandContext) {
        ctx.remove_selected_edge();
        ctx.remove_selected_node();
    }
    fn undo(&mut self, ctx: &mut crate::canvas::CommandContext) {
        for edge in &self.selected_edge {
            ctx.add_edge(edge.clone());
            ctx.add_selected_edge(edge.id, true);
        }

        for node in &self.selected_node {
            ctx.add_node(node.clone());
            ctx.add_selected_node(node.id, true);
        }
    }
}
