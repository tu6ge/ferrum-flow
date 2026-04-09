//! Read/write [`ferrum_flow::Graph`] as JSON (no GPU).

use std::path::Path;

use ferrum_flow::{Graph, PluginContext};

use crate::viewport_fit::fit_entire_graph_in_viewport;

/// Load graph from path; returns `Err` if missing or invalid.
pub fn load_graph_from_path(path: &Path) -> Result<Graph, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Graph::from_json(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

/// Save a graph as pretty-printed JSON.
pub fn save_graph_to_path(graph: &Graph, path: &Path) -> Result<(), String> {
    let text = serde_json::to_string_pretty(graph).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Load from file if present and valid, else call `default`.
pub fn load_or_default(path: &Path, default: impl FnOnce() -> Graph) -> Graph {
    match load_graph_from_path(path) {
        Ok(g) => g,
        Err(e) => {
            if path.exists() {
                eprintln!("shader-studio: {e}, using default graph");
            }
            default()
        }
    }
}

/// Replace the canvas graph (clear caches/selection, refit viewport); no undo, no disk write.
pub fn replace_canvas_graph(ctx: &mut PluginContext, g: Graph) {
    *ctx.graph = g;
    ctx.port_offset_cache_clear_all();
    ctx.history_clear();
    ctx.graph.clear_selected_node();
    ctx.graph.clear_selected_edge();
    fit_entire_graph_in_viewport(ctx);
}
