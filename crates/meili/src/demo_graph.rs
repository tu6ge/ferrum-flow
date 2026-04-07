//! Sample workflows: main pipeline (branch + merge + format) and a second mini batch line.

use ferrum_flow::{Graph, PortPosition};
use serde_json::json;

pub fn build_sample_workflow(graph: &mut Graph) {
    build_main_pipeline(graph);
    build_batch_lane(graph);
}

/// User → preprocess → orchestrator (2× branch) → merge → search → gate → format → response.
fn build_main_pipeline(graph: &mut Graph) {
    let n_in = graph
        .create_node("io_start")
        .position(40.0, 260.0)
        .size(200.0, 88.0)
        .data(json!({
            "title": "User message",
            "subtitle": "Chat · API · webhook"
        }))
        .output()
        .build(graph);
    let out_in = graph.get_node(&n_in).unwrap().outputs[0];

    let n_preprocess = graph
        .create_node("tool")
        .position(240.0, 260.0)
        .size(180.0, 84.0)
        .data(json!({
            "title": "Intent & normalize",
            "subtitle": "Schema · locale · PII trim"
        }))
        .input()
        .output()
        .build(graph);
    let in_pre = graph.get_node(&n_preprocess).unwrap().inputs[0];
    let out_pre = graph.get_node(&n_preprocess).unwrap().outputs[0];

    let n_agent = graph
        .create_node("agent")
        .position(440.0, 240.0)
        .size(220.0, 108.0)
        .data(json!({
            "title": "Orchestrator",
            "subtitle": "Plan branch · call tools · memory"
        }))
        .input()
        .output()
        .output()
        .build(graph);
    let in_agent = graph.get_node(&n_agent).unwrap().inputs[0];
    let out_agent_plan = graph.get_node(&n_agent).unwrap().outputs[0];
    let out_agent_rag = graph.get_node(&n_agent).unwrap().outputs[1];

    let n_llm = graph
        .create_node("llm")
        .position(700.0, 120.0)
        .size(200.0, 96.0)
        .data(json!({
            "title": "Reasoning model",
            "subtitle": "CoT · JSON mode · tools schema"
        }))
        .input()
        .output()
        .build(graph);
    let in_llm = graph.get_node(&n_llm).unwrap().inputs[0];
    let out_llm = graph.get_node(&n_llm).unwrap().outputs[0];

    let n_rag = graph
        .create_node("tool")
        .position(700.0, 340.0)
        .size(200.0, 92.0)
        .data(json!({
            "title": "Vector retrieve",
            "subtitle": "Embeddings · rerank · citations"
        }))
        .input_at(PortPosition::Top)
        .output_at(PortPosition::Bottom)
        .build(graph);
    let in_rag = graph.get_node(&n_rag).unwrap().inputs[0];
    let out_rag = graph.get_node(&n_rag).unwrap().outputs[0];

    let n_merge = graph
        .create_node("router")
        .position(920.0, 220.0)
        .size(200.0, 96.0)
        .data(json!({
            "title": "Merge context",
            "subtitle": "Plan + retrieved docs"
        }))
        .input()
        .input()
        .output()
        .build(graph);
    let in_merge_a = graph.get_node(&n_merge).unwrap().inputs[0];
    let in_merge_b = graph.get_node(&n_merge).unwrap().inputs[1];
    let out_merge = graph.get_node(&n_merge).unwrap().outputs[0];

    let n_search = graph
        .create_node("tool")
        .position(1140.0, 220.0)
        .size(200.0, 88.0)
        .data(json!({
            "title": "Web search",
            "subtitle": "Live fetch · snippets"
        }))
        .input()
        .output()
        .build(graph);
    let in_search = graph.get_node(&n_search).unwrap().inputs[0];
    let out_search = graph.get_node(&n_search).unwrap().outputs[0];

    let n_gate = graph
        .create_node("router")
        .position(1340.0, 220.0)
        .size(180.0, 80.0)
        .data(json!({
            "title": "Quality gate",
            "subtitle": "Retry · escalate · pass"
        }))
        .input()
        .output()
        .build(graph);
    let in_gate = graph.get_node(&n_gate).unwrap().inputs[0];
    let out_gate = graph.get_node(&n_gate).unwrap().outputs[0];

    let n_format = graph
        .create_node("llm")
        .position(1520.0, 220.0)
        .size(190.0, 88.0)
        .data(json!({
            "title": "Answer formatter",
            "subtitle": "Tone · markdown · citations list"
        }))
        .input()
        .output()
        .build(graph);
    let in_format = graph.get_node(&n_format).unwrap().inputs[0];
    let out_format = graph.get_node(&n_format).unwrap().outputs[0];

    let n_out = graph
        .create_node("io_end")
        .position(1720.0, 240.0)
        .size(200.0, 88.0)
        .data(json!({
            "title": "Response",
            "subtitle": "Stream · SSE · client"
        }))
        .input()
        .build(graph);
    let in_out = graph.get_node(&n_out).unwrap().inputs[0];

    graph
        .create_edge()
        .source(out_in)
        .target(in_pre)
        .build(graph);
    graph
        .create_edge()
        .source(out_pre)
        .target(in_agent)
        .build(graph);
    graph
        .create_edge()
        .source(out_agent_plan)
        .target(in_llm)
        .build(graph);
    graph
        .create_edge()
        .source(out_agent_rag)
        .target(in_rag)
        .build(graph);
    graph
        .create_edge()
        .source(out_llm)
        .target(in_merge_a)
        .build(graph);
    graph
        .create_edge()
        .source(out_rag)
        .target(in_merge_b)
        .build(graph);
    graph
        .create_edge()
        .source(out_merge)
        .target(in_search)
        .build(graph);
    graph
        .create_edge()
        .source(out_search)
        .target(in_gate)
        .build(graph);
    graph
        .create_edge()
        .source(out_gate)
        .target(in_format)
        .build(graph);
    graph
        .create_edge()
        .source(out_format)
        .target(in_out)
        .build(graph);
}

/// Second row: quick “batch scoring” lane to show more cards without touching the main DAG.
fn build_batch_lane(graph: &mut Graph) {
    let n_batch_in = graph
        .create_node("io_start")
        .position(40.0, 520.0)
        .size(190.0, 84.0)
        .data(json!({
            "title": "Batch prompts",
            "subtitle": "File · queue · cron"
        }))
        .output()
        .build(graph);
    let out_batch_in = graph.get_node(&n_batch_in).unwrap().outputs[0];

    let n_score = graph
        .create_node("llm")
        .position(260.0, 500.0)
        .size(200.0, 92.0)
        .data(json!({
            "title": "Score & tag",
            "subtitle": "Rubric · safety labels"
        }))
        .input()
        .output()
        .build(graph);
    let in_score = graph.get_node(&n_score).unwrap().inputs[0];
    let out_score = graph.get_node(&n_score).unwrap().outputs[0];

    let n_tool_batch = graph
        .create_node("tool")
        .position(500.0, 508.0)
        .size(190.0, 84.0)
        .data(json!({
            "title": "Export CSV",
            "subtitle": "S3 · Sheets · webhook"
        }))
        .input()
        .output()
        .build(graph);
    let in_tool_b = graph.get_node(&n_tool_batch).unwrap().inputs[0];
    let out_tool_b = graph.get_node(&n_tool_batch).unwrap().outputs[0];

    let n_stub = graph
        .create_node("")
        .position(720.0, 512.0)
        .size(176.0, 76.0)
        .data(json!({
            "title": "Review queue",
            "subtitle": "Generic step — rename / retype"
        }))
        .input()
        .output()
        .build(graph);
    let in_stub = graph.get_node(&n_stub).unwrap().inputs[0];
    let out_stub = graph.get_node(&n_stub).unwrap().outputs[0];

    let n_batch_agent = graph
        .create_node("agent")
        .position(920.0, 496.0)
        .size(210.0, 100.0)
        .data(json!({
            "title": "Post-process agent",
            "subtitle": "Dedupe · merge runs"
        }))
        .input()
        .output()
        .build(graph);
    let in_ba = graph.get_node(&n_batch_agent).unwrap().inputs[0];
    let out_ba = graph.get_node(&n_batch_agent).unwrap().outputs[0];

    let n_batch_out = graph
        .create_node("io_end")
        .position(1160.0, 518.0)
        .size(190.0, 84.0)
        .data(json!({
            "title": "Batch report",
            "subtitle": "Metrics dashboard"
        }))
        .input()
        .build(graph);
    let in_batch_out = graph.get_node(&n_batch_out).unwrap().inputs[0];

    graph
        .create_edge()
        .source(out_batch_in)
        .target(in_score)
        .build(graph);
    graph
        .create_edge()
        .source(out_score)
        .target(in_tool_b)
        .build(graph);
    graph
        .create_edge()
        .source(out_tool_b)
        .target(in_stub)
        .build(graph);
    graph
        .create_edge()
        .source(out_stub)
        .target(in_ba)
        .build(graph);
    graph
        .create_edge()
        .source(out_ba)
        .target(in_batch_out)
        .build(graph);
}
