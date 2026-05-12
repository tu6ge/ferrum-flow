//! Multiple GPUI windows: **⌘⇧G / Ctrl⇧G** runs auto-layout with different **recipes** on different
//! graphs so you can compare **layered only**, **force only**, **L→F**, **L→F→pack**, **L→pack**,
//! **F→pack**, etc. Windows with **+∅** include **isolated nodes** (no edges) to show
//! [`PackIsolatedNodesLayout`] pulling them under the connected block when
//! [`LayoutOptions::pack_isolated_nodes`] is true.
//!
//! Run: `cargo run -p ferrum-flow --example layout_windows`

use std::sync::Arc;

use ferrum_flow::{
    layout::{
        ForceDirectedLayout, LayeredDagLayout, LayoutOptions, LayoutPhase, LayoutPipeline,
        LayoutStrategy, PackIsolatedNodesLayout,
    },
    *,
};
use gpui::{
    AppContext as _, Application, Bounds, Point, Size, TitlebarOptions, WindowBounds,
    WindowOptions, px,
};
use serde_json::json;

/// Which pipeline + [`LayoutOptions`] this window uses.
#[derive(Clone, Copy)]
enum Recipe {
    LayeredOnly,
    ForceOnly,
    LayeredThenForce,
    /// Three stages; set `pack_isolated_nodes` so the post-pass runs.
    LayeredThenForcePack,
    /// Two stages: initializer + post-process strip for degree-0 nodes.
    LayeredThenPack,
    /// Force refinement then pack isolates under the force result’s bbox.
    ForceThenPack,
}

fn opts_pack_on() -> LayoutOptions {
    let mut o = LayoutOptions::default();
    o.pack_isolated_nodes = true;
    o
}

fn auto_for(recipe: Recipe) -> AutoLayoutPlugin {
    match recipe {
        Recipe::LayeredOnly => AutoLayoutPlugin::new().strategy(LayeredDagLayout),
        Recipe::ForceOnly => AutoLayoutPlugin::new().strategy(ForceDirectedLayout::default()),
        Recipe::LayeredThenForce => AutoLayoutPlugin::new().strategy(LayoutPipeline::with_meta(
            "l_f",
            "L→F",
            LayoutPhase::Optimizer,
            vec![
                Arc::new(LayeredDagLayout) as Arc<dyn LayoutStrategy>,
                Arc::new(ForceDirectedLayout::default()),
            ],
        )),
        Recipe::LayeredThenForcePack => {
            AutoLayoutPlugin::new()
                .options(opts_pack_on())
                .strategy(LayoutPipeline::with_meta(
                    "l_f_p",
                    "L→F→pack",
                    LayoutPhase::Optimizer,
                    vec![
                        Arc::new(LayeredDagLayout) as Arc<dyn LayoutStrategy>,
                        Arc::new(ForceDirectedLayout::default()),
                        Arc::new(PackIsolatedNodesLayout),
                    ],
                ))
        }
        Recipe::LayeredThenPack => {
            AutoLayoutPlugin::new()
                .options(opts_pack_on())
                .strategy(LayoutPipeline::with_meta(
                    "l_p",
                    "L→pack",
                    LayoutPhase::Optimizer,
                    vec![
                        Arc::new(LayeredDagLayout) as Arc<dyn LayoutStrategy>,
                        Arc::new(PackIsolatedNodesLayout),
                    ],
                ))
        }
        Recipe::ForceThenPack => {
            AutoLayoutPlugin::new()
                .options(opts_pack_on())
                .strategy(LayoutPipeline::with_meta(
                    "f_p",
                    "F→pack",
                    LayoutPhase::Optimizer,
                    vec![
                        Arc::new(ForceDirectedLayout::default()) as Arc<dyn LayoutStrategy>,
                        Arc::new(PackIsolatedNodesLayout),
                    ],
                ))
        }
    }
}

fn main() {
    Application::new().run(|cx| {
        let demos: Vec<(&'static str, Graph, Recipe)> = vec![
            (
                "01 layered | chain",
                graph_linear_chain(),
                Recipe::LayeredOnly,
            ),
            ("02 force | ring", graph_directed_cycle(), Recipe::ForceOnly),
            (
                "03 L→F | diamond",
                graph_diamond(),
                Recipe::LayeredThenForce,
            ),
            (
                "04 L→F→pack | diamond (no ∅)",
                graph_diamond(),
                Recipe::LayeredThenForcePack,
            ),
            (
                "05 L→F→pack | chain+∅",
                graph_chain_with_orphans(),
                Recipe::LayeredThenForcePack,
            ),
            (
                "06 L→pack | chain+∅",
                graph_chain_with_orphans(),
                Recipe::LayeredThenPack,
            ),
            (
                "07 F→pack | chain+∅",
                graph_chain_with_orphans(),
                Recipe::ForceThenPack,
            ),
            (
                "08 layered | 2 comp",
                graph_two_components(),
                Recipe::LayeredOnly,
            ),
            ("09 L→F | fan", graph_fan_out(), Recipe::LayeredThenForce),
            (
                "10 L→F→pack | fan+∅",
                graph_fan_with_orphan(),
                Recipe::LayeredThenForcePack,
            ),
            (
                "11 layered | 1 node",
                graph_single_node(),
                Recipe::LayeredOnly,
            ),
        ];

        let cols = 3;
        let col_w = 430.0;
        let row_h = 360.0;
        for (i, (title, graph, recipe)) in demos.into_iter().enumerate() {
            let col = (i % cols) as f32;
            let row = (i / cols) as f32;
            let origin = Point::new(px(24.0 + col * col_w), px(24.0 + row * row_h));
            let size = Size::new(px(400.0), px(300.0));
            let opts = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(origin, size))),
                titlebar: Some(TitlebarOptions {
                    title: Some(title.into()),
                    ..Default::default()
                }),
                focus: i == 0,
                ..Default::default()
            };

            cx.open_window(opts, |window, cx| {
                let auto = auto_for(recipe);
                cx.new(|ctx| {
                    FlowCanvas::builder(graph, ctx, window)
                        .default_plugins()
                        .plugin(FitAllGraphPlugin::new())
                        .plugin(ToastPlugin::new())
                        .plugin(auto)
                        .build()
                })
            })
            .unwrap();
        }
    });
}

fn graph_linear_chain() -> Graph {
    Graph::build(|g| {
        let (_, _, o0) = g
            .create_node("")
            .position(20.0, 120.0)
            .output()
            .data(json!({ "label": "A" }))
            .build_with_ports();
        let (_, i1, o1) = g
            .create_node("")
            .position(200.0, 200.0)
            .input()
            .output()
            .data(json!({ "label": "B" }))
            .build_with_ports();
        let (_, i2, o2) = g
            .create_node("")
            .position(400.0, 80.0)
            .input()
            .output()
            .data(json!({ "label": "C" }))
            .build_with_ports();
        let (_, i3, _) = g
            .create_node("")
            .position(600.0, 240.0)
            .input()
            .data(json!({ "label": "D" }))
            .build_with_ports();
        g.create_edge().source(o0[0]).target(i1[0]).build();
        g.create_edge().source(o1[0]).target(i2[0]).build();
        g.create_edge().source(o2[0]).target(i3[0]).build();
    })
}

/// A→B→C plus two nodes with no edges (good for **pack** demos).
fn graph_chain_with_orphans() -> Graph {
    Graph::build(|g| {
        let (_, _, oa) = g
            .create_node("")
            .position(20.0, 120.0)
            .size(72.0, 36.0)
            .output()
            .data(json!({ "label": "A" }))
            .build_with_ports();
        let (_, ib, ob) = g
            .create_node("")
            .position(180.0, 120.0)
            .size(72.0, 36.0)
            .input()
            .output()
            .data(json!({ "label": "B" }))
            .build_with_ports();
        let (_, ic, _) = g
            .create_node("")
            .position(340.0, 120.0)
            .size(72.0, 36.0)
            .input()
            .data(json!({ "label": "C" }))
            .build_with_ports();
        g.create_edge().source(oa[0]).target(ib[0]).build();
        g.create_edge().source(ob[0]).target(ic[0]).build();

        g.create_node("")
            .position(480.0, 30.0)
            .size(64.0, 32.0)
            .data(json!({ "label": "∅1" }))
            .build();
        g.create_node("")
            .position(500.0, 220.0)
            .size(64.0, 32.0)
            .data(json!({ "label": "∅2" }))
            .build();
    })
}

fn graph_directed_cycle() -> Graph {
    Graph::build(|g| {
        let (_, ia, oa) = g
            .create_node("")
            .position(100.0, 100.0)
            .input()
            .output()
            .data(json!({ "label": "A" }))
            .build_with_ports();
        let (_, ib, ob) = g
            .create_node("")
            .position(300.0, 100.0)
            .input()
            .output()
            .data(json!({ "label": "B" }))
            .build_with_ports();
        let (_, ic, oc) = g
            .create_node("")
            .position(200.0, 260.0)
            .input()
            .output()
            .data(json!({ "label": "C" }))
            .build_with_ports();
        g.create_edge().source(oa[0]).target(ib[0]).build();
        g.create_edge().source(ob[0]).target(ic[0]).build();
        g.create_edge().source(oc[0]).target(ia[0]).build();
    })
}

fn graph_diamond() -> Graph {
    Graph::build(|g| {
        let (_, _, oa) = g
            .create_node("")
            .position(80.0, 140.0)
            .output()
            .data(json!({ "label": "A" }))
            .build_with_ports();
        let (_, ib, ob) = g
            .create_node("")
            .position(240.0, 80.0)
            .input()
            .output()
            .data(json!({ "label": "B" }))
            .build_with_ports();
        let (_, ic, oc) = g
            .create_node("")
            .position(240.0, 220.0)
            .input()
            .output()
            .data(json!({ "label": "C" }))
            .build_with_ports();
        let (_, idd, _) = g
            .create_node("")
            .position(420.0, 140.0)
            .input()
            .input()
            .data(json!({ "label": "D" }))
            .build_with_ports();
        g.create_edge().source(oa[0]).target(ib[0]).build();
        g.create_edge().source(oa[0]).target(ic[0]).build();
        g.create_edge().source(ob[0]).target(idd[0]).build();
        g.create_edge().source(oc[0]).target(idd[1]).build();
    })
}

fn graph_two_components() -> Graph {
    Graph::build(|g| {
        let (_, _, o1) = g
            .create_node("")
            .position(40.0, 80.0)
            .output()
            .data(json!({ "label": "L1" }))
            .build_with_ports();
        let (_, i2, _) = g
            .create_node("")
            .position(200.0, 80.0)
            .input()
            .data(json!({ "label": "L2" }))
            .build_with_ports();
        let (_, _, o3) = g
            .create_node("")
            .position(40.0, 220.0)
            .output()
            .data(json!({ "label": "R1" }))
            .build_with_ports();
        let (_, i4, _) = g
            .create_node("")
            .position(200.0, 220.0)
            .input()
            .data(json!({ "label": "R2" }))
            .build_with_ports();
        g.create_edge().source(o1[0]).target(i2[0]).build();
        g.create_edge().source(o3[0]).target(i4[0]).build();
    })
}

fn graph_single_node() -> Graph {
    Graph::build(|g| {
        g.create_node("")
            .position(120.0, 120.0)
            .output()
            .data(json!({ "label": "only" }))
            .build();
    })
}

fn graph_fan_out() -> Graph {
    Graph::build(|g| {
        let (_, _, o0) = g
            .create_node("")
            .position(60.0, 140.0)
            .output()
            .data(json!({ "label": "src" }))
            .build_with_ports();
        let (_, i1, _) = g
            .create_node("")
            .position(280.0, 60.0)
            .input()
            .data(json!({ "label": "t1" }))
            .build_with_ports();
        let (_, i2, _) = g
            .create_node("")
            .position(280.0, 150.0)
            .input()
            .data(json!({ "label": "t2" }))
            .build_with_ports();
        let (_, i3, _) = g
            .create_node("")
            .position(280.0, 240.0)
            .input()
            .data(json!({ "label": "t3" }))
            .build_with_ports();
        g.create_edge().source(o0[0]).target(i1[0]).build();
        g.create_edge().source(o0[0]).target(i2[0]).build();
        g.create_edge().source(o0[0]).target(i3[0]).build();
    })
}

/// Fan-out plus one isolated node (far from the fan).
fn graph_fan_with_orphan() -> Graph {
    Graph::build(|g| {
        let (_, _, o0) = g
            .create_node("")
            .position(40.0, 130.0)
            .output()
            .data(json!({ "label": "src" }))
            .build_with_ports();
        let (_, i1, _) = g
            .create_node("")
            .position(220.0, 50.0)
            .input()
            .data(json!({ "label": "t1" }))
            .build_with_ports();
        let (_, i2, _) = g
            .create_node("")
            .position(220.0, 130.0)
            .input()
            .data(json!({ "label": "t2" }))
            .build_with_ports();
        let (_, i3, _) = g
            .create_node("")
            .position(220.0, 210.0)
            .input()
            .data(json!({ "label": "t3" }))
            .build_with_ports();
        g.create_edge().source(o0[0]).target(i1[0]).build();
        g.create_edge().source(o0[0]).target(i2[0]).build();
        g.create_edge().source(o0[0]).target(i3[0]).build();

        g.create_node("")
            .position(420.0, 130.0)
            .size(56.0, 30.0)
            .data(json!({ "label": "∅" }))
            .build();
    })
}
