//! Multiple GPUI windows, each with a different graph. **⌘⇧G / Ctrl⇧G** runs auto-layout: most
//! windows use [`LayeredDagLayout`]; the **directed cycle** demo uses [`ForceDirectedLayout`] so
//! you can compare DAG layering vs. force on a ring.
//!
//! Run: `cargo run -p ferrum-flow --example layout_windows`

use ferrum_flow::{
    layout::{ForceDirectedLayout, LayeredDagLayout},
    *,
};
use gpui::{
    AppContext as _, Application, Bounds, Point, Size, TitlebarOptions, WindowBounds,
    WindowOptions, px,
};
use serde_json::json;

fn main() {
    Application::new().run(|cx| {
        #[derive(Clone, Copy)]
        enum LayoutKind {
            Layered,
            Force,
        }

        let demos: Vec<(&'static str, Graph, LayoutKind)> = vec![
            ("1: linear chain", graph_linear_chain(), LayoutKind::Layered),
            (
                "2: directed cycle (force)",
                graph_directed_cycle(),
                LayoutKind::Force,
            ),
            ("3: diamond DAG", graph_diamond(), LayoutKind::Layered),
            ("4: two components", graph_two_components(), LayoutKind::Layered),
            ("5: single node", graph_single_node(), LayoutKind::Layered),
            ("6: fan-out", graph_fan_out(), LayoutKind::Layered),
        ];

        let col_w = 440.0;
        let row_h = 380.0;
        for (i, (title, graph, layout_kind)) in demos.into_iter().enumerate() {
            let col = (i % 3) as f32;
            let row = (i / 3) as f32;
            let origin = Point::new(px(32.0 + col * col_w), px(28.0 + row * row_h));
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
                cx.new(|ctx| {
                    let auto = match layout_kind {
                        LayoutKind::Layered => {
                            AutoLayoutPlugin::new().strategy(LayeredDagLayout)
                        }
                        LayoutKind::Force => {
                            AutoLayoutPlugin::new().strategy(ForceDirectedLayout::default())
                        }
                    };
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
