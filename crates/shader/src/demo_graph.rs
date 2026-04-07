use std::sync::atomic::{AtomicUsize, Ordering};

use ferrum_flow::{Graph, NodeId};
use serde_json::json;

pub type ShaderDemoBuilder = fn() -> Graph;

/// Built-in samples in order (top-left Samples menu).
pub static SHADER_STUDIO_DEMOS: &[(&str, ShaderDemoBuilder)] = &[
    ("Aurora", sample_shader_graph),
    ("Empty", empty_shader_graph),
    ("Simple gradient", radial_gradient_demo_graph),
    ("Post (vignette)", postprocess_vignette_demo_graph),
];

static DEMO_CURSOR: AtomicUsize = AtomicUsize::new(0);

/// Select sample by index (modulo `SHADER_STUDIO_DEMOS.len()`) and sync cursor.
pub fn shader_demo_select(index: usize) -> (&'static str, Graph) {
    let n = SHADER_STUDIO_DEMOS.len();
    let i = index % n;
    DEMO_CURSOR.store(i, Ordering::Relaxed);
    let (title, f) = SHADER_STUDIO_DEMOS[i];
    (title, f())
}

/// Advance to next sample (wrap) and return title + graph.
pub fn shader_demo_advance() -> (&'static str, Graph) {
    let n = SHADER_STUDIO_DEMOS.len();
    let cur = DEMO_CURSOR.load(Ordering::Relaxed);
    let i = (cur + 1) % n;
    shader_demo_select(i)
}

fn wire(graph: &mut Graph, from: NodeId, out_i: usize, to: NodeId, in_i: usize) {
    let s = graph.get_node(&from).unwrap().outputs[out_i];
    let t = graph.get_node(&to).unwrap().inputs[in_i];
    graph.create_edge().source(s).target(t).build(graph);
}

fn scalar(graph: &mut Graph, x: f32, y: f32, label: &str, value: f32) -> NodeId {
    graph
        .create_node("scalar")
        .position(x, y)
        .size(136.0, 62.0)
        .output()
        .data(json!({ "label": label, "value": value }))
        .build(graph)
}

/// Aurora bands: time-driven sine scales noise, smoothstep bands, two-color mix; radial |UV−center| for extensions.
pub fn sample_shader_graph() -> Graph {
    let mut graph = Graph::new();

    let uv = graph
        .create_node("uv")
        .position(48.0, 72.0)
        .size(160.0, 72.0)
        .output()
        .data(json!({ "label": "UV", "hint": "0‥1 screen space" }))
        .build(&mut graph);

    let s05a = scalar(&mut graph, 48.0, 200.0, "½", 0.5);
    let s05b = scalar(&mut graph, 48.0, 292.0, "½", 0.5);

    let center = graph
        .create_node("join_ff")
        .position(248.0, 228.0)
        .size(168.0, 80.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "Screen center", "hint": "vec2(0.5,0.5)" }))
        .build(&mut graph);

    let delta = graph
        .create_node("sub_vec2")
        .position(456.0, 188.0)
        .size(172.0, 80.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "UV − center", "hint": "radial" }))
        .build(&mut graph);

    let _r_len = graph
        .create_node("length_v2")
        .position(664.0, 200.0)
        .size(156.0, 72.0)
        .input()
        .output()
        .data(json!({ "label": "|ΔUV|", "hint": "vignette hook" }))
        .build(&mut graph);

    let time = graph
        .create_node("time")
        .position(48.0, 420.0)
        .size(160.0, 72.0)
        .output()
        .data(json!({ "label": "Time", "hint": "uniform" }))
        .build(&mut graph);

    let sin_w = graph
        .create_node("sin_f")
        .position(248.0, 428.0)
        .size(148.0, 68.0)
        .input()
        .output()
        .data(json!({ "label": "sin(t)", "hint": "domain warp" }))
        .build(&mut graph);

    let s4 = scalar(&mut graph, 48.0, 540.0, "×4", 4.0);
    let s8 = scalar(&mut graph, 48.0, 632.0, "+8", 8.0);

    let mul_amp = graph
        .create_node("mul_ff")
        .position(428.0, 412.0)
        .size(152.0, 72.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "wave ×4", "hint": "f·f" }))
        .build(&mut graph);

    let scale_fac = graph
        .create_node("add_ff")
        .position(612.0, 412.0)
        .size(152.0, 72.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "freq", "hint": "8+4sin" }))
        .build(&mut graph);

    let uv_scaled = graph
        .create_node("mul_vec2_f")
        .position(812.0, 108.0)
        .size(184.0, 84.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "Warped UV", "hint": "uv·freq" }))
        .build(&mut graph);

    let noise_n = graph
        .create_node("noise")
        .position(1028.0, 112.0)
        .size(180.0, 88.0)
        .input()
        .output()
        .data(json!({ "label": "Noise", "hint": "hash sample" }))
        .build(&mut graph);

    let low_s = scalar(&mut graph, 900.0, 272.0, "lo", 0.22);
    let high_s = scalar(&mut graph, 900.0, 364.0, "hi", 0.68);

    let band = graph
        .create_node("smoothstep")
        .position(1068.0, 292.0)
        .size(196.0, 96.0)
        .input()
        .input()
        .input()
        .output()
        .data(json!({ "label": "Aurora bands", "hint": "smoothstep" }))
        .build(&mut graph);

    let col_a = graph
        .create_node("color")
        .position(1172.0, 32.0)
        .size(196.0, 82.0)
        .output()
        .data(json!({ "label": "Deep", "hint": "vec3 #1a0a2e" }))
        .build(&mut graph);

    let col_b = graph
        .create_node("color")
        .position(1172.0, 472.0)
        .size(196.0, 82.0)
        .output()
        .data(json!({ "label": "Glow", "hint": "vec3 #ff6b9d" }))
        .build(&mut graph);

    let mix_rgb = graph
        .create_node("mix")
        .position(1328.0, 252.0)
        .size(208.0, 108.0)
        .input()
        .input()
        .input()
        .output()
        .data(json!({ "label": "Blend", "hint": "mix(a,b,t)" }))
        .build(&mut graph);

    let frag = graph
        .create_node("output")
        .position(1576.0, 264.0)
        .size(208.0, 92.0)
        .input()
        .data(json!({ "label": "Fragment", "hint": "@location(0) vec4" }))
        .build(&mut graph);

    wire(&mut graph, s05a, 0, center, 0);
    wire(&mut graph, s05b, 0, center, 1);
    wire(&mut graph, uv, 0, delta, 0);
    wire(&mut graph, center, 0, delta, 1);
    wire(&mut graph, delta, 0, _r_len, 0);

    wire(&mut graph, time, 0, sin_w, 0);
    wire(&mut graph, sin_w, 0, mul_amp, 0);
    wire(&mut graph, s4, 0, mul_amp, 1);
    wire(&mut graph, mul_amp, 0, scale_fac, 0);
    wire(&mut graph, s8, 0, scale_fac, 1);

    wire(&mut graph, uv, 0, uv_scaled, 0);
    wire(&mut graph, scale_fac, 0, uv_scaled, 1);

    wire(&mut graph, uv_scaled, 0, noise_n, 0);

    wire(&mut graph, low_s, 0, band, 0);
    wire(&mut graph, high_s, 0, band, 1);
    wire(&mut graph, noise_n, 0, band, 2);

    wire(&mut graph, band, 0, mix_rgb, 0);
    wire(&mut graph, col_a, 0, mix_rgb, 1);
    wire(&mut graph, col_b, 0, mix_rgb, 2);

    wire(&mut graph, mix_rgb, 0, frag, 0);

    graph
}

/// Minimal **[color] → [output]** neutral gray (empty graph still compiles).
pub fn empty_shader_graph() -> Graph {
    let mut graph = Graph::new();

    let fill = graph
        .create_node("color")
        .position(120.0, 96.0)
        .size(176.0, 72.0)
        .output()
        .data(json!({ "label": "Neutral gray", "value": [0.42, 0.44, 0.48] }))
        .build(&mut graph);

    let frag = graph
        .create_node("output")
        .position(384.0, 96.0)
        .size(200.0, 88.0)
        .input()
        .data(json!({ "label": "Fragment", "hint": "vec3 to vec4" }))
        .build(&mut graph);

    wire(&mut graph, fill, 0, frag, 0);
    graph
}

/// Radial gradient: |UV−center| → smoothstep → two-color mix.
pub fn radial_gradient_demo_graph() -> Graph {
    let mut graph = Graph::new();

    let uv = graph
        .create_node("uv")
        .position(40.0, 100.0)
        .size(152.0, 68.0)
        .output()
        .data(json!({ "label": "UV" }))
        .build(&mut graph);

    let s05a = scalar(&mut graph, 40.0, 220.0, "½", 0.5);
    let s05b = scalar(&mut graph, 40.0, 312.0, "½", 0.5);

    let center = graph
        .create_node("join_ff")
        .position(232.0, 252.0)
        .size(160.0, 76.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "center" }))
        .build(&mut graph);

    let delta = graph
        .create_node("sub_vec2")
        .position(432.0, 96.0)
        .size(168.0, 76.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "Δuv" }))
        .build(&mut graph);

    let rad = graph
        .create_node("length_v2")
        .position(632.0, 104.0)
        .size(148.0, 68.0)
        .input()
        .output()
        .data(json!({ "label": "|Δ|" }))
        .build(&mut graph);

    let e0 = scalar(&mut graph, 232.0, 400.0, "lo", 0.0);
    let e1 = scalar(&mut graph, 232.0, 492.0, "hi", 0.72);

    let t = graph
        .create_node("smoothstep")
        .position(632.0, 252.0)
        .size(184.0, 88.0)
        .input()
        .input()
        .input()
        .output()
        .data(json!({ "label": "falloff" }))
        .build(&mut graph);

    let col_in = graph
        .create_node("color")
        .position(824.0, 40.0)
        .size(180.0, 76.0)
        .output()
        .data(json!({ "label": "Inner", "value": [0.95, 0.92, 0.82] }))
        .build(&mut graph);

    let col_out = graph
        .create_node("color")
        .position(824.0, 372.0)
        .size(180.0, 76.0)
        .output()
        .data(json!({ "label": "Outer", "value": [0.12, 0.18, 0.38] }))
        .build(&mut graph);

    let blend = graph
        .create_node("mix")
        .position(1040.0, 180.0)
        .size(200.0, 100.0)
        .input()
        .input()
        .input()
        .output()
        .data(json!({ "label": "mix" }))
        .build(&mut graph);

    let frag = graph
        .create_node("output")
        .position(1288.0, 196.0)
        .size(200.0, 88.0)
        .input()
        .data(json!({ "label": "out" }))
        .build(&mut graph);

    wire(&mut graph, s05a, 0, center, 0);
    wire(&mut graph, s05b, 0, center, 1);
    wire(&mut graph, uv, 0, delta, 0);
    wire(&mut graph, center, 0, delta, 1);
    wire(&mut graph, delta, 0, rad, 0);
    wire(&mut graph, e0, 0, t, 0);
    wire(&mut graph, e1, 0, t, 1);
    wire(&mut graph, rad, 0, t, 2);
    wire(&mut graph, t, 0, blend, 0);
    wire(&mut graph, col_in, 0, blend, 1);
    wire(&mut graph, col_out, 0, blend, 2);
    wire(&mut graph, blend, 0, frag, 0);

    graph
}

/// Short post chain: noise grade + radial vignette (smoothstep × scene).
pub fn postprocess_vignette_demo_graph() -> Graph {
    let mut graph = Graph::new();

    let uv = graph
        .create_node("uv")
        .position(32.0, 96.0)
        .size(152.0, 68.0)
        .output()
        .data(json!({ "label": "UV" }))
        .build(&mut graph);

    let n = graph
        .create_node("noise")
        .position(224.0, 96.0)
        .size(176.0, 80.0)
        .input()
        .output()
        .data(json!({ "label": "grain" }))
        .build(&mut graph);

    let cool = graph
        .create_node("color")
        .position(224.0, 220.0)
        .size(176.0, 72.0)
        .output()
        .data(json!({ "label": "Shadow tint", "value": [0.15, 0.22, 0.42] }))
        .build(&mut graph);

    let warm = graph
        .create_node("color")
        .position(224.0, 328.0)
        .size(176.0, 72.0)
        .output()
        .data(json!({ "label": "Highlight tint", "value": [0.98, 0.72, 0.38] }))
        .build(&mut graph);

    let graded = graph
        .create_node("mix")
        .position(448.0, 180.0)
        .size(200.0, 100.0)
        .input()
        .input()
        .input()
        .output()
        .data(json!({ "label": "Color grade" }))
        .build(&mut graph);

    let s05a = scalar(&mut graph, 448.0, 336.0, "½", 0.5);
    let s05b = scalar(&mut graph, 448.0, 428.0, "½", 0.5);

    let center = graph
        .create_node("join_ff")
        .position(448.0, 540.0)
        .size(160.0, 76.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "ctr" }))
        .build(&mut graph);

    let delta = graph
        .create_node("sub_vec2")
        .position(688.0, 368.0)
        .size(168.0, 76.0)
        .input()
        .input()
        .output()
        .data(json!({ "label": "Δ" }))
        .build(&mut graph);

    let rad = graph
        .create_node("length_v2")
        .position(888.0, 376.0)
        .size(148.0, 68.0)
        .input()
        .output()
        .data(json!({ "label": "r" }))
        .build(&mut graph);

    let v0 = scalar(&mut graph, 688.0, 512.0, "v0", 0.85);
    let v1 = scalar(&mut graph, 688.0, 604.0, "v1", 0.28);

    let vig = graph
        .create_node("smoothstep")
        .position(888.0, 520.0)
        .size(192.0, 92.0)
        .input()
        .input()
        .input()
        .output()
        .data(json!({ "label": "vignette" }))
        .build(&mut graph);

    let black = graph
        .create_node("color")
        .position(1088.0, 96.0)
        .size(168.0, 72.0)
        .output()
        .data(json!({ "label": "Crush black", "value": [0.02, 0.02, 0.05] }))
        .build(&mut graph);

    let comp = graph
        .create_node("mix")
        .position(1088.0, 240.0)
        .size(208.0, 104.0)
        .input()
        .input()
        .input()
        .output()
        .data(json!({ "label": "Composite" }))
        .build(&mut graph);

    let frag = graph
        .create_node("output")
        .position(1344.0, 264.0)
        .size(200.0, 88.0)
        .input()
        .data(json!({ "label": "Fragment" }))
        .build(&mut graph);

    wire(&mut graph, uv, 0, n, 0);
    wire(&mut graph, n, 0, graded, 0);
    wire(&mut graph, cool, 0, graded, 1);
    wire(&mut graph, warm, 0, graded, 2);

    wire(&mut graph, s05a, 0, center, 0);
    wire(&mut graph, s05b, 0, center, 1);
    wire(&mut graph, uv, 0, delta, 0);
    wire(&mut graph, center, 0, delta, 1);
    wire(&mut graph, delta, 0, rad, 0);

    wire(&mut graph, v0, 0, vig, 0);
    wire(&mut graph, v1, 0, vig, 1);
    wire(&mut graph, rad, 0, vig, 2);

    wire(&mut graph, vig, 0, comp, 0);
    wire(&mut graph, black, 0, comp, 1);
    wire(&mut graph, graded, 0, comp, 2);
    wire(&mut graph, comp, 0, frag, 0);

    graph
}
