//! Flow canvas with [`ExecutionHighlightPlugin`]: custom node bodies show **1**, **+1**, and the
//! **result** (empty until you press **F5** to run the graph).

use ferrum_flow::*;
use ferrum_flow_executor::{
    ExecuteGraphEvent, ExecutionHighlightPlugin, ExecutorContext, GraphExecutor, NodeOutput,
    NodeProcessor, NodeRegistry, PortValues,
};
use gpui::{
    AnyElement, AppContext as _, Application, Element as _, ParentElement as _, Styled,
    WindowOptions, div, px, rgb, white,
};
use serde_json::{Value, json};

struct SourceNode;

impl NodeProcessor for SourceNode {
    fn name(&self) -> &str {
        "demo_source"
    }

    fn execute(&self, node: &Node, _ctx: &mut ExecutorContext) -> anyhow::Result<NodeOutput> {
        let mut outputs = PortValues::new();
        if let Some(p) = node.outputs.first() {
            outputs.insert(*p, json!(1.0));
        }
        Ok(NodeOutput {
            node_id: node.id,
            outputs,
            error: None,
        })
    }
}

struct AddOneNode;

impl NodeProcessor for AddOneNode {
    fn name(&self) -> &str {
        "demo_add_one"
    }

    fn execute(&self, node: &Node, ctx: &mut ExecutorContext) -> anyhow::Result<NodeOutput> {
        let v = node
            .inputs
            .first()
            .and_then(|p| ctx.get_input(p))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            + 1.0;
        let mut outputs = PortValues::new();
        if let Some(p) = node.outputs.first() {
            outputs.insert(*p, json!(v));
        }
        Ok(NodeOutput {
            node_id: node.id,
            outputs,
            error: None,
        })
    }
}

struct SinkNode;

impl NodeProcessor for SinkNode {
    fn name(&self) -> &str {
        "demo_sink"
    }

    fn execute(&self, node: &Node, _ctx: &mut ExecutorContext) -> anyhow::Result<NodeOutput> {
        let _ = node.inputs.first();
        Ok(NodeOutput {
            node_id: node.id,
            outputs: PortValues::new(),
            error: None,
        })
    }
}

/// Renders `node.data["text"]` inside the node body (ports use the default layout hook).
struct CalcDemoRenderer;

impl NodeRenderer for CalcDemoRenderer {
    fn render(&self, node: &Node, ctx: &mut RenderContext) -> AnyElement {
        let text = node.data.get("text").and_then(|v| v.as_str()).unwrap_or("");

        let screen = ctx.world_to_screen(node.point());
        let node_id = node.id;
        let selected = ctx.graph.selected_node.contains(&node_id);

        div()
            .absolute()
            .left(screen.x)
            .top(screen.y)
            .w(node.size.width * ctx.viewport.zoom)
            .h(node.size.height * ctx.viewport.zoom)
            .bg(rgb(0x2D3142))
            .rounded(px(8.0))
            .border(px(2.0))
            .border_color(rgb(if selected { 0xFF7800 } else { 0x5A6078 }))
            .p(px(12.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .h_full()
                    .child(div().child(text.to_string()).text_color(white())),
            )
            .into_any()
    }

    fn port_render(&self, node: &Node, port: &Port, ctx: &mut RenderContext) -> Option<AnyElement> {
        let frame = ctx.port_screen_frame(node, port)?;
        Some(
            frame
                .anchor_div()
                .rounded_full()
                .border(px(1.0))
                .border_color(rgb(0x1A192B))
                .bg(rgb(0xE8EAEF))
                .into_any(),
        )
    }
}

/// Emits [`ExecuteGraphEvent`] when the user presses F5 (canvas must be focused).
struct F5RunGraphPlugin;

impl Plugin for F5RunGraphPlugin {
    fn name(&self) -> &'static str {
        "f5_run_graph"
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if let FlowEvent::Input(InputEvent::KeyDown(ev)) = event {
            if ev.keystroke.key == "f5" {
                ctx.emit(FlowEvent::custom(ExecuteGraphEvent));
                return EventResult::Stop;
            }
        }
        EventResult::Continue
    }

    fn priority(&self) -> i32 {
        200
    }
}

/// Builds three linked nodes; returns the sink node id and its input port for writing the result.
fn build_demo_graph(graph: &mut Graph) -> (NodeId, PortId) {
    let n1 = graph
        .create_node("calc_demo")
        .execute_type("demo_source")
        .position(72.0, 120.0)
        .size(152.0, 88.0)
        .data(json!({ "text": "1" }))
        .output()
        .build(graph);
    let out1 = graph.get_node(&n1).unwrap().outputs[0];

    let n2 = graph
        .create_node("calc_demo")
        .execute_type("demo_add_one")
        .position(268.0, 120.0)
        .size(152.0, 88.0)
        .data(json!({ "text": "+1" }))
        .input()
        .output()
        .build(graph);
    let in2 = graph.get_node(&n2).unwrap().inputs[0];
    let out2 = graph.get_node(&n2).unwrap().outputs[0];

    let n3 = graph
        .create_node("calc_demo")
        .execute_type("demo_sink")
        .position(464.0, 120.0)
        .size(152.0, 88.0)
        .data(json!({ "text": "" }))
        .input_at(PortPosition::Top)
        .build(graph);
    let in3 = graph.get_node(&n3).unwrap().inputs[0];

    graph.create_edge().source(out1).target(in2).build(graph);
    graph.create_edge().source(out2).target(in3).build(graph);

    (n3, in3)
}

fn format_result_value(v: &Value) -> String {
    if let Some(n) = v.as_f64() {
        if n.fract() == 0.0 {
            format!("{}", n as i64)
        } else {
            n.to_string()
        }
    } else {
        v.to_string()
    }
}

fn main() {
    eprintln!("Executor demo — custom node labels: 1, +1, (empty) → press F5 with canvas focused.");

    let mut registry = NodeRegistry::new();
    registry.register(SourceNode);
    registry.register(AddOneNode);
    registry.register(SinkNode);

    let executor = GraphExecutor::new(registry);

    Application::new().run(|cx| {
        let mut graph = Graph::new();
        let (sink_id, sink_in) = build_demo_graph(&mut graph);

        let exec_plugin = ExecutionHighlightPlugin::new(executor)
            .with_step_delay(std::time::Duration::from_millis(400))
            .with_on_run_complete(move |g, exec_ctx| {
                let text = exec_ctx
                    .get_input(&sink_in)
                    .map(format_result_value)
                    .unwrap_or_default();
                if let Some(n) = g.get_node_mut(&sink_id) {
                    n.data = json!({ "text": text });
                }
            });

        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .plugin(F5RunGraphPlugin)
                    .plugin(exec_plugin)
                    .plugin(SelectionPlugin::new())
                    .plugin(NodeInteractionPlugin::new())
                    .plugin(ViewportPlugin::new())
                    .plugin(ZoomControlsPlugin::new())
                    .plugin(FocusSelectionPlugin::new())
                    .plugin(FitAllGraphPlugin::new())
                    .plugin(BackgroundPlugin::new())
                    .plugin(NodePlugin::new())
                    .plugin(PortInteractionPlugin::new())
                    .plugin(EdgePlugin::new())
                    .plugin(ClipboardPlugin::new())
                    .plugin(ContextMenuPlugin::new())
                    .plugin(DeletePlugin::new())
                    .plugin(HistoryPlugin::new())
                    .node_renderer("calc_demo", CalcDemoRenderer)
                    .build()
            })
        })
        .unwrap();
    });
}
