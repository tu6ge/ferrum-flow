use ferrum_flow::FlowTheme;

/// Dark canvas theme for node editing / shader graphs.
pub fn shader_studio_theme() -> FlowTheme {
    let mut t = FlowTheme::default();
    t.node_card_background = 0x00252830;
    t.node_card_border = 0x0049505a;
    t.node_card_border_selected = 0x00FFB86B;
    t.undefined_node_background = 0x00302828;
    t.undefined_node_border = 0x00FF6B6B;
    t.node_caption_text = 0x00e8eaed;
    t.undefined_node_caption_text = 0x00b8bcc4;
    t.default_port_fill = 0x007dd3fc;
    t.background = 0x00101820;
    t.background_grid_dot = 0x00334055;
    t.edge_stroke = 0x00607080;
    t.edge_stroke_selected = 0x00FFB86B;
    t.selection_rect_border = 0x0078A0FF;
    t.selection_rect_fill_rgba = 0x78A0FF33;
    t.port_preview_line = 0x0090a0b8;
    t.port_preview_dot = 0x007dd3fc;
    t.minimap_background = 0x00182028;
    t.minimap_border = 0x00405060;
    t.minimap_edge = 0x00607080;
    t.minimap_node_fill = 0x00354050;
    t.minimap_node_stroke = 0x00708090;
    t.minimap_viewport_stroke = 0x0078a0ff;
    t.zoom_controls_background = 0x00252830;
    t.zoom_controls_border = 0x0049505a;
    t.zoom_controls_text = 0x00e8eaed;
    t.context_menu_background = 0x00252830;
    t.context_menu_border = 0x0049505a;
    t.context_menu_text = 0x00e8eaed;
    t.context_menu_shortcut_text = 0x009098a8;
    t.context_menu_separator = 0x00384048;
    t
}
