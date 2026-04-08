//! Default FlowCanvas plugins and node renderers for the shader studio (keeps `main` small).

use ferrum_flow::*;

use crate::ShaderNodeRenderer;
use crate::demo_menu::DemoMenuPlugin;
use crate::wgpu_preview::WgpuPreviewPlugin;
use crate::wgsl_preview::WgslPreviewPlugin;

pub fn shader_studio_extra_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(DemoMenuPlugin::new()),
        Box::new(WgslPreviewPlugin::new()),
        Box::new(WgpuPreviewPlugin::new()),
        Box::new(MinimapPlugin::new()),
        Box::new(ZoomControlsPlugin::new()),
        Box::new(ClipboardPlugin::new()),
        Box::new(ContextMenuPlugin::new()),
        Box::new(SelectAllViewportPlugin::new()),
        Box::new(AlignPlugin::new()),
        Box::new(FocusSelectionPlugin::new()),
        Box::new(FitAllGraphPlugin::new()),
    ]
}

pub fn shader_studio_node_renderers() -> Vec<(String, Box<dyn NodeRenderer>)> {
    [
        "uv",
        "time",
        "scalar",
        "join_ff",
        "sub_vec2",
        "length_v2",
        "sin_f",
        "mul_ff",
        "add_ff",
        "mul_vec2_f",
        "smoothstep",
        "noise",
        "color",
        "mix",
        "output",
    ]
    .into_iter()
    .map(|name| {
        (
            name.to_string(),
            Box::new(ShaderNodeRenderer) as Box<dyn NodeRenderer>,
        )
    })
    .collect()
}
