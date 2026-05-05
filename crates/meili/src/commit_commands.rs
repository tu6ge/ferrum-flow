//! Shell → canvas graph updates as [`ferrum_flow::Command`]s (history + sync `to_ops`).

use ferrum_flow::{
    Command, CommandContext, CompositeCommand, CreateEdge, CreateNode, CreatePort, GraphOp,
    PortKind, PortPosition,
};
use gpui::SharedString;

use crate::pick_state;
use crate::plugins::node_kind_preset::{NodeKindPreset, preset_for_digit};
use crate::plugins::pick_link_event::PickNodeTypeForPendingLink;

fn with_opposite_port<'a>(
    kind: PortKind,
    preset: NodeKindPreset,
    b: ferrum_flow::NodeBuilder<'a>,
) -> ferrum_flow::NodeBuilder<'a> {
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

pub(crate) fn build_add_node_composite(
    ctx: &mut CommandContext,
    label: &str,
    world_x: f32,
    world_y: f32,
    kind_digit: u8,
) -> Option<CompositeCommand> {
    let preset = preset_for_digit(kind_digit).or_else(|| preset_for_digit(7))?;
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
    let (new_node, new_ports, _) = builder.build_raw();
    let mut comp = CompositeCommand::new();
    comp.push(CreateNode::new(new_node));
    for port in new_ports {
        comp.push(CreatePort::new(port));
    }
    Some(comp)
}

pub(crate) fn build_pending_link_composite(
    ctx: &mut CommandContext,
    p: PickNodeTypeForPendingLink,
    choice: NodeKindPreset,
) -> Option<CompositeCommand> {
    let source = ctx.graph.get_port(&p.source_port).cloned()?;
    let x: f32 = p.end_world.x.into();
    let y: f32 = p.end_world.y.into();

    let (node_type, w, h, data) = choice.describe();

    let mut builder = ctx.create_node(node_type);
    builder = builder
        .position(x, y)
        .size(w, h)
        .data(data)
        .execute_type(node_type);
    builder = with_opposite_port(source.kind(), choice, builder);

    let (new_node, new_ports, _) = builder.build_raw();

    let edge = match source.kind() {
        PortKind::Output => {
            let in_port = *new_node.inputs().first()?;
            ctx.new_edge().source(p.source_port).target(in_port)
        }
        PortKind::Input => {
            let out_port = *new_node.outputs().first()?;
            ctx.new_edge().source(out_port).target(p.source_port)
        }
    };

    let mut comp = CompositeCommand::new();
    comp.push(CreateNode::new(new_node));
    for port in new_ports {
        comp.push(CreatePort::new(port));
    }
    comp.push(CreateEdge::new(edge));
    Some(comp)
}

/// Confirms the bottom-bar type picker after a dangling link ([`crate::shell::MeiliShell`]).
pub struct NodeTypeSelectConfirmCommand {
    digit: u8,
    inner: Option<CompositeCommand>,
}

impl NodeTypeSelectConfirmCommand {
    pub fn new(digit: u8) -> Self {
        Self { digit, inner: None }
    }
}

impl Command for NodeTypeSelectConfirmCommand {
    fn name(&self) -> &'static str {
        "meili_node_type_select_confirm"
    }

    fn execute(&mut self, ctx: &mut CommandContext) {
        let Some(pending) = pick_state::pending_take() else {
            return;
        };
        let Some(preset) = preset_for_digit(self.digit) else {
            return;
        };
        let Some(mut comp) = build_pending_link_composite(ctx, pending, preset) else {
            return;
        };
        comp.execute(ctx);
        self.inner = Some(comp);
    }

    fn undo(&mut self, ctx: &mut CommandContext) {
        if let Some(ref mut c) = self.inner {
            c.undo(ctx);
        }
    }

    fn to_ops(&self, ctx: &mut CommandContext) -> Vec<GraphOp> {
        let Some(pending) = pick_state::pending_take() else {
            return vec![];
        };
        let Some(preset) = preset_for_digit(self.digit) else {
            return vec![];
        };
        let Some(comp) = build_pending_link_composite(ctx, pending, preset) else {
            return vec![];
        };
        comp.to_ops(ctx)
    }
}

/// Confirms the “Add node” dialog ([`crate::shell::MeiliShell`]).
pub struct AddNodeConfirmCommand {
    label: SharedString,
    world_x: f32,
    world_y: f32,
    kind_digit: u8,
    inner: Option<CompositeCommand>,
}

impl AddNodeConfirmCommand {
    pub fn new(label: SharedString, world_x: f32, world_y: f32, kind_digit: u8) -> Self {
        Self {
            label,
            world_x,
            world_y,
            kind_digit,
            inner: None,
        }
    }
}

impl Command for AddNodeConfirmCommand {
    fn name(&self) -> &'static str {
        "meili_add_node_confirm"
    }

    fn execute(&mut self, ctx: &mut CommandContext) {
        let s = self.label.trim();
        if s.is_empty() {
            return;
        }
        let Some(mut comp) =
            build_add_node_composite(ctx, s, self.world_x, self.world_y, self.kind_digit)
        else {
            return;
        };
        comp.execute(ctx);
        self.inner = Some(comp);
    }

    fn undo(&mut self, ctx: &mut CommandContext) {
        if let Some(ref mut c) = self.inner {
            c.undo(ctx);
        }
    }

    fn to_ops(&self, ctx: &mut CommandContext) -> Vec<GraphOp> {
        let s = self.label.trim();
        if s.is_empty() {
            return vec![];
        }
        let Some(comp) =
            build_add_node_composite(ctx, s, self.world_x, self.world_y, self.kind_digit)
        else {
            return vec![];
        };
        comp.to_ops(ctx)
    }
}
