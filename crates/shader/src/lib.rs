//! Node-based shader graph demo on `ferrum-flow`: sample graphs, node chrome, JSON persist, validation.

pub mod demo_graph;
mod demo_menu;
pub mod graph_io;
pub mod persist_plugin;
mod preview_rig;
pub mod shader_render;
pub mod studio;
pub mod theme;
pub mod validate;
mod viewport_fit;
mod wgpu_preview;
pub mod wgsl_codegen;
mod wgsl_preview;

pub use demo_graph::{SHADER_STUDIO_DEMOS, sample_shader_graph, shader_demo_select};
pub use demo_menu::DemoMenuPlugin;
pub use graph_io::{
    load_graph_from_path, load_or_default, replace_canvas_graph, save_graph_to_path,
};
pub use persist_plugin::ShaderGraphFilePlugin;
pub use shader_render::ShaderNodeRenderer;
pub use studio::{shader_studio_extra_plugins, shader_studio_node_renderers};
pub use theme::shader_studio_theme;
pub use validate::graph_validation_notes;
pub use wgpu_preview::WgpuPreviewPlugin;
pub use wgsl_codegen::{CompileError, compile_graph_to_wgsl};
pub use wgsl_preview::WgslPreviewPlugin;
