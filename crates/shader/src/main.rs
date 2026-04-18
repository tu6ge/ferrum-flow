use std::path::{Path, PathBuf};

use ferrum_flow::*;
use gpui::{AppContext as _, Application, WindowOptions};
use shader::{
    ShaderGraphFilePlugin, load_or_default, sample_shader_graph, shader_studio_extra_plugins,
    shader_studio_node_renderers, shader_studio_theme,
};

fn graph_file_path() -> PathBuf {
    std::env::var("SHADER_STUDIO_GRAPH")
        .map(PathBuf::from)
        .ok()
        .or_else(|| std::env::args().nth(1).map(PathBuf::from))
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("shader_graph.json"))
}

fn main() {
    let graph_path = graph_file_path();
    eprintln!(
        "shader-studio: {:?} — Samples menu (top-left), ⌘S / ⌘O, toasts, WGSL + GPU preview (right)",
        graph_path
    );

    Application::new().run(move |cx| {
        let path = graph_path.clone();
        let graph = load_or_default(&path, sample_shader_graph);

        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|ctx| {
                FlowCanvas::builder(graph, ctx, window)
                    .theme(shader_studio_theme())
                    .default_plugins()
                    .plugin(ShaderGraphFilePlugin::new(path.clone()))
                    .plugins(shader_studio_extra_plugins())
                    .node_renderers(shader_studio_node_renderers())
                    .build()
            })
        })
        .unwrap();
    });
}
