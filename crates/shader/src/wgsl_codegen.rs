//! Graph → WGSL: reverse topological order from `output`, emit a fullscreen module for naga/wgpu.
//!
//! Supported node types are listed in `compile_graph_to_wgsl` (`match`).

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use ferrum_flow::{Graph, NodeId, PortId};

#[derive(Debug, Clone)]
pub struct CompileError(pub String);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for CompileError {}

fn err(s: impl Into<String>) -> CompileError {
    CompileError(s.into())
}

fn find_output_nodes(graph: &Graph) -> Vec<NodeId> {
    graph
        .nodes()
        .iter()
        .filter(|(_, n)| n.node_type == "output")
        .map(|(id, _)| *id)
        .collect()
}

/// Follow edges from a target input port to the upstream output; error if unwired.
fn source_port_for_input(graph: &Graph, input_port: PortId) -> Result<PortId, CompileError> {
    graph
        .edges
        .values()
        .find(|e| e.target_port == input_port)
        .map(|e| e.source_port)
        .ok_or_else(|| err(format!("input port {input_port:?} is not connected")))
}

fn owner_of_port(graph: &Graph, port: PortId) -> Result<NodeId, CompileError> {
    graph
        .ports
        .get(&port)
        .map(|p| p.node_id)
        .ok_or_else(|| err(format!("unknown port {port:?}")))
}

/// Post-order walk from `output` along inputs: emission order from sources to sink.
fn dependency_order(graph: &Graph, output: NodeId) -> Result<Vec<NodeId>, CompileError> {
    let mut visited = HashSet::new();
    let mut post = Vec::new();

    fn visit(
        graph: &Graph,
        id: NodeId,
        visited: &mut HashSet<NodeId>,
        post: &mut Vec<NodeId>,
    ) -> Result<(), CompileError> {
        if !visited.insert(id) {
            return Ok(());
        }
        let node = graph
            .get_node(&id)
            .ok_or_else(|| err(format!("node {id:?} is missing")))?;
        for in_p in &node.inputs {
            let src = source_port_for_input(graph, *in_p)?;
            let up = owner_of_port(graph, src)?;
            visit(graph, up, visited, post)?;
        }
        post.push(id);
        Ok(())
    }

    visit(graph, output, &mut visited, &mut post)?;
    Ok(post)
}

fn var_for_port<'a>(
    port_to_var: &'a HashMap<PortId, String>,
    p: PortId,
) -> Result<&'a str, CompileError> {
    port_to_var
        .get(&p)
        .map(|s| s.as_str())
        .ok_or_else(|| err(format!("port {p:?} has no generated variable (order/type issue)")))
}

fn vec3_literal_from_node_data(data: &serde_json::Value) -> String {
    if let Some(arr) = data.get("value").and_then(|v| v.as_array()) {
        if arr.len() >= 3 {
            let r = arr[0].as_f64().unwrap_or(0.0) as f32;
            let g = arr[1].as_f64().unwrap_or(0.0) as f32;
            let b = arr[2].as_f64().unwrap_or(0.0) as f32;
            return format!("vec3<f32>({r}, {g}, {b})");
        }
    }
    if let Some(hint) = data.get("hint").and_then(|v| v.as_str()) {
        if let Some((r, g, b)) = parse_hex_rgb(hint) {
            return format!("vec3<f32>({r}, {g}, {b})");
        }
    }
    // Default colors aligned with common UI labels
    if data
        .get("label")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s.contains("Tint A"))
    {
        return "vec3<f32>(0.49, 0.83, 0.99)".to_string();
    }
    if data
        .get("label")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s.contains("Tint B"))
    {
        return "vec3<f32>(1.0, 0.72, 0.42)".to_string();
    }
    if data
        .get("label")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s == "Deep")
    {
        return "vec3<f32>(0.102, 0.039, 0.18)".to_string();
    }
    if data
        .get("label")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s == "Glow")
    {
        return "vec3<f32>(1.0, 0.42, 0.616)".to_string();
    }
    "vec3<f32>(0.5, 0.5, 0.5)".to_string()
}

fn parse_hex_rgb(s: &str) -> Option<(f32, f32, f32)> {
    let i = s.find('#')?;
    let hex = s[i + 1..].trim_start_matches('#');
    let hex = hex.get(..6)?;
    let r = u8::from_str_radix(hex.get(0..2)?, 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(hex.get(2..4)?, 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(hex.get(4..6)?, 16).ok()? as f32 / 255.0;
    Some((r, g, b))
}

/// Compile the editor graph to WGSL for `naga` / `wgpu` (includes `vs_main` + `fs_main`).
pub fn compile_graph_to_wgsl(graph: &Graph) -> Result<String, CompileError> {
    let outs = find_output_nodes(graph);
    match outs.len() {
        0 => return Err(err("graph has no node with node_type == \"output\"")),
        1 => {}
        _ => return Err(err("only exactly one output node is supported")),
    }
    let output_id = outs[0];
    let order = dependency_order(graph, output_id)?;

    let needs_noise = order.iter().any(|id| {
        graph
            .get_node(id)
            .is_some_and(|n| n.node_type == "noise")
    });

    let mut port_to_var: HashMap<PortId, String> = HashMap::new();
    let mut body = String::new();
    let mut n = 0u32;

    for &nid in &order {
        let node = graph.get_node(&nid).ok_or_else(|| err("node missing"))?;

        match node.node_type.as_str() {
            "output" => {
                if node.inputs.len() != 1 {
                    return Err(err("output node must have exactly 1 input"));
                }
                let src_p = source_port_for_input(graph, node.inputs[0])?;
                let v = var_for_port(&port_to_var, src_p)?;
                writeln!(&mut body, "    return vec4<f32>({v}, 1.0);").unwrap();
            }
            "uv" => {
                if !node.outputs.is_empty() {
                    let name = format!("v{n}");
                    n += 1;
                    writeln!(&mut body, "    let {name} = in.uv;").unwrap();
                    port_to_var.insert(node.outputs[0], name);
                }
            }
            "time" => {
                if !node.outputs.is_empty() {
                    let name = format!("v{n}");
                    n += 1;
                    writeln!(&mut body, "    let {name} = globals.time;").unwrap();
                    port_to_var.insert(node.outputs[0], name);
                }
            }
            "color" => {
                if node.outputs.len() != 1 {
                    return Err(err("color node must have exactly 1 output"));
                }
                let name = format!("v{n}");
                n += 1;
                let lit = vec3_literal_from_node_data(&node.data);
                writeln!(&mut body, "    let {name} = {lit};").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "noise" => {
                if node.inputs.len() != 1 || node.outputs.len() != 1 {
                    return Err(err("noise node must have 1 input and 1 output"));
                }
                let src_p = source_port_for_input(graph, node.inputs[0])?;
                let uv = var_for_port(&port_to_var, src_p)?;
                let name = format!("v{n}");
                n += 1;
                writeln!(&mut body, "    let {name} = ff_value_noise({uv});").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "mix" => {
                if node.inputs.len() != 3 || node.outputs.len() != 1 {
                    return Err(err("mix node must have 3 inputs and 1 output (t, a, b)"));
                }
                let p_t = source_port_for_input(graph, node.inputs[0])?;
                let p_a = source_port_for_input(graph, node.inputs[1])?;
                let p_b = source_port_for_input(graph, node.inputs[2])?;
                let t = var_for_port(&port_to_var, p_t)?;
                let a = var_for_port(&port_to_var, p_a)?;
                let b = var_for_port(&port_to_var, p_b)?;
                let name = format!("v{n}");
                n += 1;
                writeln!(&mut body, "    let {name} = mix({a}, {b}, {t});").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "scalar" => {
                if node.inputs.is_empty() && node.outputs.len() == 1 {
                    let name = format!("v{n}");
                    n += 1;
                    let v = node
                        .data
                        .get("value")
                        .and_then(|x| x.as_f64())
                        .unwrap_or(0.0) as f32;
                    // WGSL float literal suffix; bare `0` breaks smoothstep overload resolution.
                    writeln!(&mut body, "    let {name} = {v}f;").unwrap();
                    port_to_var.insert(node.outputs[0], name);
                } else {
                    return Err(err("scalar node must have 0 inputs, 1 output, numeric data.value"));
                }
            }
            "join_ff" => {
                if node.inputs.len() != 2 || node.outputs.len() != 1 {
                    return Err(err("join_ff must be 2× f32 → vec2"));
                }
                let p0 = source_port_for_input(graph, node.inputs[0])?;
                let p1 = source_port_for_input(graph, node.inputs[1])?;
                let a = var_for_port(&port_to_var, p0)?;
                let b = var_for_port(&port_to_var, p1)?;
                let name = format!("v{n}");
                n += 1;
                writeln!(&mut body, "    let {name} = vec2<f32>({a}, {b});").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "sub_vec2" => {
                if node.inputs.len() != 2 || node.outputs.len() != 1 {
                    return Err(err("sub_vec2 must be 2× vec2 → vec2"));
                }
                let p0 = source_port_for_input(graph, node.inputs[0])?;
                let p1 = source_port_for_input(graph, node.inputs[1])?;
                let a = var_for_port(&port_to_var, p0)?;
                let b = var_for_port(&port_to_var, p1)?;
                let name = format!("v{n}");
                n += 1;
                writeln!(&mut body, "    let {name} = {a} - {b};").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "length_v2" => {
                if node.inputs.len() != 1 || node.outputs.len() != 1 {
                    return Err(err("length_v2 must be vec2 → f32"));
                }
                let p0 = source_port_for_input(graph, node.inputs[0])?;
                let a = var_for_port(&port_to_var, p0)?;
                let name = format!("v{n}");
                n += 1;
                writeln!(&mut body, "    let {name} = length({a});").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "sin_f" => {
                if node.inputs.len() != 1 || node.outputs.len() != 1 {
                    return Err(err("sin_f must be f32 → f32"));
                }
                let p0 = source_port_for_input(graph, node.inputs[0])?;
                let a = var_for_port(&port_to_var, p0)?;
                let name = format!("v{n}");
                n += 1;
                writeln!(&mut body, "    let {name} = sin({a});").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "mul_ff" | "add_ff" => {
                if node.inputs.len() != 2 || node.outputs.len() != 1 {
                    return Err(err("mul_ff / add_ff must be 2× f32 → f32"));
                }
                let p0 = source_port_for_input(graph, node.inputs[0])?;
                let p1 = source_port_for_input(graph, node.inputs[1])?;
                let a = var_for_port(&port_to_var, p0)?;
                let b = var_for_port(&port_to_var, p1)?;
                let name = format!("v{n}");
                n += 1;
                let op = if node.node_type == "mul_ff" { '*' } else { '+' };
                writeln!(&mut body, "    let {name} = {a} {op} {b};").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "mul_vec2_f" => {
                if node.inputs.len() != 2 || node.outputs.len() != 1 {
                    return Err(err("mul_vec2_f must be vec2 × f32 → vec2"));
                }
                let p0 = source_port_for_input(graph, node.inputs[0])?;
                let p1 = source_port_for_input(graph, node.inputs[1])?;
                let a = var_for_port(&port_to_var, p0)?;
                let b = var_for_port(&port_to_var, p1)?;
                let name = format!("v{n}");
                n += 1;
                writeln!(&mut body, "    let {name} = {a} * {b};").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            "smoothstep" => {
                if node.inputs.len() != 3 || node.outputs.len() != 1 {
                    return Err(err("smoothstep must be 3× f32 → f32 (edge0, edge1, x)"));
                }
                let p0 = source_port_for_input(graph, node.inputs[0])?;
                let p1 = source_port_for_input(graph, node.inputs[1])?;
                let p2 = source_port_for_input(graph, node.inputs[2])?;
                let e0 = var_for_port(&port_to_var, p0)?;
                let e1 = var_for_port(&port_to_var, p1)?;
                let x = var_for_port(&port_to_var, p2)?;
                let name = format!("v{n}");
                n += 1;
                writeln!(&mut body, "    let {name} = smoothstep({e0}, {e1}, {x});").unwrap();
                port_to_var.insert(node.outputs[0], name);
            }
            other => {
                return Err(err(format!(
                    "unsupported node type {other:?} (see wgsl_codegen match arms)"
                )));
            }
        }
    }

    let noise_fn = if needs_noise {
        r#"
fn ff_value_noise(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}
"#
    } else {
        ""
    };

    let wgsl = format!(
        r#"struct Globals {{
    resolution: vec2<f32>,
    time: f32,
    _pad: f32,
}}

@group(0) @binding(0) var<uniform> globals: Globals;

struct VsOut {{
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {{
    var tri = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    let p = tri[vertex_index];
    var o: VsOut;
    o.clip_pos = vec4<f32>(p, 0.0, 1.0);
    o.uv = p * 0.5 + vec2<f32>(0.5, 0.5);
    return o;
}}
{noise_fn}
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {{
{body}
}}
"#
    );

    Ok(wgsl)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo_graph::SHADER_STUDIO_DEMOS;
    use crate::sample_shader_graph;

    #[test]
    fn sample_graph_dependency_chain_ends_at_output() {
        let g = sample_shader_graph();
        let order = dependency_order(&g, find_output_nodes(&g)[0]).unwrap();
        assert_eq!(
            g.get_node(order.last().unwrap()).unwrap().node_type,
            "output"
        );
        assert!(order.len() >= 10);
    }

    #[test]
    fn sample_graph_emits_wgsl_with_expected_calls() {
        let wgsl = compile_graph_to_wgsl(&sample_shader_graph()).unwrap();
        assert!(wgsl.contains("@vertex"));
        assert!(wgsl.contains("@fragment"));
        assert!(wgsl.contains("fn fs_main"));
        assert!(wgsl.contains("ff_value_noise"));
        assert!(wgsl.contains("smoothstep"));
        assert!(wgsl.contains("sin("));
        assert!(wgsl.contains("mix("));
        assert!(wgsl.contains("return vec4<f32>"));
    }

    #[test]
    fn sample_graph_wgsl_parses_with_naga() {
        let wgsl = compile_graph_to_wgsl(&sample_shader_graph()).unwrap();
        let module = naga::front::wgsl::parse_str(&wgsl);
        if let Err(ref e) = module {
            panic!("naga parse error: {e}");
        }
        assert!(module.unwrap().entry_points.len() >= 2);
    }

    #[test]
    fn all_builtin_shader_demos_compile_and_parse() {
        for (title, build) in SHADER_STUDIO_DEMOS {
            let g = build();
            let wgsl = compile_graph_to_wgsl(&g).unwrap_or_else(|e| {
                panic!("compile {title}: {e}");
            });
            naga::front::wgsl::parse_str(&wgsl).unwrap_or_else(|e| {
                panic!("naga {title}: {e}");
            });
        }
    }
}
