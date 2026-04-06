//! Dark “agent studio” palette (inspired by modern LLM / automation canvases).
//!
//! [`apply_flow_chrome`] pushes these values into [`ferrum_flow::FlowTheme`] so core chrome
//! (edges, selection, minimap, zoom bar, context menu, port preview) matches the canvas.

use ferrum_flow::FlowTheme;

pub const CANVAS_BG: u32 = 0x0a0e14;
pub const GRID_DOT: u32 = 0x3d4f66;
pub const TEXT_PRIMARY: u32 = 0xe8ecf1;
pub const TEXT_MUTED: u32 = 0x8b98a8;

pub const PORT_IN: u32 = 0x22d3ee;
pub const PORT_OUT: u32 = 0xe879f9;
pub const PORT_RING: u32 = 0x0a0e14;

/// Top bar fill ([`gpui::rgba`] `RRGGBBAA`).
pub const HUD_BAR_BG_RGBA: u32 = 0x0a0e14cc;
/// Top bar bottom edge ([`gpui::rgba`]).
pub const HUD_BAR_BORDER_RGBA: u32 = 0xffffff12;

pub fn accent_agent() -> u32 {
    0x8b7cf7
}
pub fn accent_llm() -> u32 {
    0x4a9eff
}
pub fn accent_tool() -> u32 {
    0x3ddc84
}
pub fn accent_router() -> u32 {
    0xffb74d
}
pub fn accent_io_in() -> u32 {
    0x7cb342
}
pub fn accent_io_out() -> u32 {
    0xff7043
}

pub const SELECTION_BORDER: u32 = 0xc4b5fd;

/// Sync core [`FlowTheme`] with the Meili palette (register [`crate::plugins::MeiliThemePlugin`]
/// first so every frame uses these tokens).
pub fn apply_flow_chrome(t: &mut FlowTheme) {
    t.background = CANVAS_BG;
    t.background_grid_dot = GRID_DOT;

    t.node_card_background = 0x00161c22;
    t.node_card_border = 0x003d4f66;
    t.node_card_border_selected = SELECTION_BORDER;
    t.undefined_node_background = 0x0018202e;
    t.undefined_node_border = 0x00ff9800;
    t.node_caption_text = TEXT_PRIMARY;
    // Used by [`crate::renderers::WorkflowNodeRenderer`] for subtitle lines.
    t.undefined_node_caption_text = TEXT_MUTED;
    t.default_port_fill = PORT_RING;

    t.edge_stroke = 0x004a5568;
    t.edge_stroke_selected = SELECTION_BORDER;
    t.selection_rect_border = SELECTION_BORDER;
    t.selection_rect_fill_rgba = 0xc4b5fd33;

    t.port_preview_line = 0x004a5568;
    // Softer than `accent_llm()` so the pending-link endpoint is visible but not neon on dark bg.
    t.port_preview_dot = 0x005f7a94;

    t.minimap_background = 0x000d1218;
    t.minimap_border = 0x003d4f66;
    t.minimap_edge = 0x004a5568;
    t.minimap_node_fill = 0x00161c22;
    t.minimap_node_stroke = 0x00586880;
    t.minimap_viewport_stroke = SELECTION_BORDER;

    t.zoom_controls_background = 0x00101824;
    t.zoom_controls_border = 0x003d4f66;
    t.zoom_controls_text = TEXT_PRIMARY;

    t.context_menu_background = 0x00101824;
    t.context_menu_border = 0x003d4f66;
    t.context_menu_text = TEXT_PRIMARY;
    t.context_menu_shortcut_text = TEXT_MUTED;
    t.context_menu_separator = 0x00283848;
}
