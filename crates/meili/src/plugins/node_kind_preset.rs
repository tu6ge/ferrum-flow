//! Shared node-kind presets for [`super::node_type_picker::NodeTypePickerPlugin`] and [`super::add_node::MeiliAddNodePlugin`].

use ferrum_flow::NodeBuilder;
use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub(crate) enum NodeKindPreset {
    Agent,
    Llm,
    Tool,
    Router,
    IoStart,
    IoEnd,
    Step,
}

pub(crate) fn preset_for_digit(d: u8) -> Option<NodeKindPreset> {
    match d {
        1 => Some(NodeKindPreset::Agent),
        2 => Some(NodeKindPreset::Llm),
        3 => Some(NodeKindPreset::Tool),
        4 => Some(NodeKindPreset::Router),
        5 => Some(NodeKindPreset::IoStart),
        6 => Some(NodeKindPreset::IoEnd),
        7 => Some(NodeKindPreset::Step),
        _ => None,
    }
}

impl NodeKindPreset {
    pub(crate) fn describe(&self) -> (&'static str, f32, f32, Value) {
        match self {
            Self::Agent => (
                "agent",
                220.0,
                104.0,
                json!({
                    "title": "Agent",
                    "subtitle": "Orchestrator"
                }),
            ),
            Self::Llm => (
                "llm",
                200.0,
                96.0,
                json!({
                    "title": "LLM",
                    "subtitle": "Model pass"
                }),
            ),
            Self::Tool => (
                "tool",
                200.0,
                88.0,
                json!({
                    "title": "Tool",
                    "subtitle": "Action"
                }),
            ),
            Self::Router => (
                "router",
                180.0,
                80.0,
                json!({
                    "title": "Router",
                    "subtitle": "Branching"
                }),
            ),
            Self::IoStart => (
                "io_start",
                200.0,
                88.0,
                json!({
                    "title": "Start",
                    "subtitle": "Input"
                }),
            ),
            Self::IoEnd => (
                "io_end",
                200.0,
                88.0,
                json!({
                    "title": "End",
                    "subtitle": "Output"
                }),
            ),
            Self::Step => (
                "",
                200.0,
                88.0,
                json!({
                    "title": "Step",
                    "subtitle": "Generic"
                }),
            ),
        }
    }

    /// Default [`Self::describe`] data with `title` replaced by the user-entered label.
    pub(crate) fn describe_with_title(&self, title: &str) -> (&'static str, f32, f32, Value) {
        let (nt, w, h, mut data) = self.describe();
        if let Some(obj) = data.as_object_mut() {
            obj.insert("title".to_string(), json!(title));
        }
        (nt, w, h, data)
    }

    /// Ports when creating a node from the add dialog (no dangling edge).
    pub(crate) fn apply_standalone_ports(self, b: NodeBuilder) -> NodeBuilder {
        match self {
            Self::IoStart => b.output(),
            Self::IoEnd => b.input(),
            _ => b.input().output(),
        }
    }
}
