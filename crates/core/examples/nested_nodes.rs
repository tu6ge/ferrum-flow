//! Nested parent/child nodes: local positions, [`Graph::paint_order`], click-to-front, port wiring.
//!
//! - **L1 Parent** at world (200, 120) with overlapping **L2** children and a nested **L2 Sub group**.
//! - **L3 Grandchild** inside Sub group (three-level hierarchy).
//! - **Root peer** overlaps the parent to exercise root-level z-order.
//! - Intra-parent (L2): Child A output → Child B input.
//! - Cross (L2→L3 subtree): Child B output → Grandchild input.
//! - Cross (L2→root): Child B output → Root peer input ([`PortScope::Boundary`]).
//!
//! Try:
//! - Click the overlapping children — the selected child should come to the front (siblings + ancestors).
//! - Press **I** — toast with current `paint_order` labels.
//! - Press **H** — toggle the HUD.
//!
//! Run: `cargo run -p ferrum-flow --example nested_nodes`

use ferrum_flow::*;
use gpui::{
    AnyElement, AppContext as _, Application, Bounds, Element as _, ParentElement as _, Size,
    Styled, WindowBounds, WindowOptions, div, px, rgb, white,
};
use serde_json::json;

fn main() {
    Application::new().run(|cx| {
        let graph = build_nested_demo_graph();

        let win_size = Size::new(px(720.0), px(520.0));
        let window_opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, win_size, cx))),
            ..Default::default()
        };

        cx.open_window(window_opts, |window, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .default_plugins()
                    .plugin(FitAllGraphPlugin::new())
                    .plugin(NestedNodesDemoPlugin::new())
                    .build()
            })
        })
        .unwrap();
    });
}

fn build_nested_demo_graph() -> Result<Graph, GraphError> {
    Graph::try_build(|g| {
        let parent = g
            .create_node("default")
            .position(200.0, 120.0)
            .size(420.0, 320.0)
            .data(json!({ "label": "Parent (L1)" }))
            .build();

        let (child_a, _, outs_a) = g
            .create_node("default")
            .position(32.0, 48.0)
            .size(180.0, 88.0)
            .output()
            .data(json!({ "label": "Child A (back)" }))
            .build_with_ports();

        let (child_b, ins_b, outs_b) = g
            .create_node("default")
            .position(120.0, 72.0)
            .size(180.0, 88.0)
            .input()
            .output_port(PortSpec::output(PortPosition::Right).with_scope(PortScope::Boundary))
            .data(json!({ "label": "Child B (front)" }))
            .build_with_ports();

        let sub_group = g
            .create_node("default")
            .position(24.0, 168.0)
            .size(280.0, 120.0)
            .data(json!({ "label": "Sub group (L2)" }))
            .build();

        let (grandchild, ins_gc, _) = g
            .create_node("default")
            .position(20.0, 40.0)
            .size(150.0, 56.0)
            .input_port(PortSpec::input(PortPosition::Left).with_scope(PortScope::Boundary))
            .output()
            .data(json!({ "label": "Grandchild (L3)" }))
            .build_with_ports();

        g.add_child(parent, child_a)?;
        g.add_child(parent, child_b)?;
        g.add_child(parent, sub_group)?;
        g.add_child(sub_group, grandchild)?;

        // Intra L2 (same parent): Child A → Child B.
        g.create_edge().source(outs_a[0]).target(ins_b[0]).build();

        // Cross L2 subtrees (Child B under Parent, Grandchild under Sub group).
        g.create_edge().source(outs_b[0]).target(ins_gc[0]).build();

        // Root sibling drawn after parent → on top where they overlap.
        let (_root_peer, ins_peer, _) = g
            .create_node("default")
            .position(160.0, 140.0)
            .size(200.0, 72.0)
            .input_port(PortSpec::input(PortPosition::Left).with_scope(PortScope::Boundary))
            .data(json!({ "label": "Root peer" }))
            .build_with_ports();

        // Cross-parent: group child → root leaf (top overlay, not inside Parent group).
        g.create_edge()
            .source(outs_b[0])
            .target(ins_peer[0])
            .build();

        Ok(())
    })
}

fn node_label(graph: &Graph, id: &NodeId) -> String {
    graph
        .get_node(id)
        .and_then(|n| n.data_ref().get("label").and_then(|v| v.as_str()))
        .unwrap_or("?")
        .to_string()
}

fn paint_order_summary(graph: &Graph) -> String {
    graph
        .paint_order()
        .iter()
        .map(|id| node_label(graph, id))
        .collect::<Vec<_>>()
        .join(" → ")
}

struct NestedNodesDemoPlugin {
    show_hud: bool,
}

impl NestedNodesDemoPlugin {
    fn new() -> Self {
        Self { show_hud: true }
    }
}

impl Plugin for NestedNodesDemoPlugin {
    fn name(&self) -> &'static str {
        "nested_nodes_demo"
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if let FlowEvent::Input(InputEvent::KeyDown(ev)) = event {
            if ev.keystroke.key == "h" {
                self.show_hud = !self.show_hud;
                ctx.notify();
                return EventResult::Stop;
            }
            if ev.keystroke.key == "i" {
                ctx.emit(FlowEvent::success(format!(
                    "paint_order: {}",
                    paint_order_summary(&ctx.graph)
                )));
                return EventResult::Stop;
            }
        }

        EventResult::Continue
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<AnyElement> {
        if !self.show_hud {
            return None;
        }

        let order_preview = paint_order_summary(ctx.graph);

        Some(
            div()
                .absolute()
                .left(px(12.0))
                .top(px(12.0))
                .px_3()
                .py_2()
                .rounded(px(8.0))
                .bg(rgb(0x001F2937))
                .text_color(white())
                .child(div().text_sm().child("Nested nodes demo"))
                .child(
                    div()
                        .text_sm()
                        .child("Click overlapping Child A / B — selection brings to front"),
                )
                .child(div().text_sm().child("L3: Parent → Sub group → Grandchild"))
                .child(
                    div()
                        .text_sm()
                        .child("Cross: Child B → Grandchild; Child B → Root peer"),
                )
                .child(div().text_sm().child("Press I: toast paint_order"))
                .child(div().text_sm().child("Press H: hide this panel"))
                .child(
                    div()
                        .text_xs()
                        .mt_1()
                        .child(format!("paint_order: {order_preview}")),
                )
                .into_any(),
        )
    }

    fn priority(&self) -> i32 {
        120
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }
}
