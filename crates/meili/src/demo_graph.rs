//! Sample agent pipeline: input → agent → model → tool → output.

use ferrum_flow::{Graph, PortPosition};
use serde_json::json;

pub fn build_sample_workflow(graph: &mut Graph) {
    let n_in = graph
        .create_node("io_start")
        .position(80.0, 220.0)
        .size(200.0, 88.0)
        .data(json!({
            "title": "User message",
            "subtitle": "Chat · API · webhook"
        }))
        .output()
        .build(graph);
    let out_in = graph.get_node(&n_in).unwrap().outputs[0];

    let n_agent = graph
        .create_node("agent")
        .position(340.0, 200.0)
        .size(220.0, 104.0)
        .data(json!({
            "title": "Orchestrator",
            "subtitle": "ReAct · memory · tools"
        }))
        .input()
        .output()
        .build(graph);
    let in_agent = graph.get_node(&n_agent).unwrap().inputs[0];
    let out_agent = graph.get_node(&n_agent).unwrap().outputs[0];

    let n_llm = graph
        .create_node("llm")
        .position(620.0, 160.0)
        .size(200.0, 96.0)
        .data(json!({
            "title": "GPT-4.1",
            "subtitle": "Reasoning pass"
        }))
        .input()
        .output()
        .build(graph);
    let in_llm = graph.get_node(&n_llm).unwrap().inputs[0];
    let out_llm = graph.get_node(&n_llm).unwrap().outputs[0];

    let n_tool = graph
        .create_node("tool")
        .position(620.0, 300.0)
        .size(200.0, 88.0)
        .data(json!({
            "title": "Search",
            "subtitle": "Web · RAG · code exec"
        }))
        .input_at(PortPosition::Top)
        .output_at(PortPosition::Bottom)
        .build(graph);
    let in_tool = graph.get_node(&n_tool).unwrap().inputs[0];
    let out_tool = graph.get_node(&n_tool).unwrap().outputs[0];

    let n_route = graph
        .create_node("router")
        .position(880.0, 230.0)
        .size(180.0, 80.0)
        .data(json!({
            "title": "Route",
            "subtitle": "Success / retry / handoff"
        }))
        .input()
        .output()
        .build(graph);
    let in_route = graph.get_node(&n_route).unwrap().inputs[0];
    let out_route = graph.get_node(&n_route).unwrap().outputs[0];

    let n_out = graph
        .create_node("io_end")
        .position(1120.0, 220.0)
        .size(200.0, 88.0)
        .data(json!({
            "title": "Response",
            "subtitle": "Stream to client"
        }))
        .input()
        .build(graph);
    let in_out = graph.get_node(&n_out).unwrap().inputs[0];

    graph
        .create_dege()
        .source(out_in)
        .target(in_agent)
        .build(graph);
    graph
        .create_dege()
        .source(out_agent)
        .target(in_llm)
        .build(graph);
    graph
        .create_dege()
        .source(out_llm)
        .target(in_tool)
        .build(graph);
    graph
        .create_dege()
        .source(out_tool)
        .target(in_route)
        .build(graph);
    graph
        .create_dege()
        .source(out_route)
        .target(in_out)
        .build(graph);
}
