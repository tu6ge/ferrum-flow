use ferrum_flow::FlowTheme;

/// Dark canvas theme for node editing / shader graphs.
pub fn shader_studio_theme() -> FlowTheme {
    FlowTheme {
        node_card_background: 0x00252830,
        node_card_border: 0x0049505a,
        node_card_border_selected: 0x00FFB86B,
        undefined_node_background: 0x00302828,
        undefined_node_border: 0x00FF6B6B,
        node_caption_text: 0x00e8eaed,
        undefined_node_caption_text: 0x00b8bcc4,
        default_port_fill: 0x007dd3fc,
        background: 0x00101820,
        background_grid_dot: 0x00334055,
        edge_stroke: 0x00607080,
        edge_stroke_selected: 0x00FFB86B,
        selection_rect_border: 0x0078A0FF,
        selection_rect_fill_rgba: 0x78A0FF33,
        port_preview_line: 0x0090a0b8,
        port_preview_dot: 0x007dd3fc,
        minimap_background: 0x00182028,
        minimap_border: 0x00405060,
        minimap_edge: 0x00607080,
        minimap_node_fill: 0x00354050,
        minimap_node_stroke: 0x00708090,
        minimap_viewport_stroke: 0x0078a0ff,
        zoom_controls_background: 0x00252830,
        zoom_controls_border: 0x0049505a,
        zoom_controls_text: 0x00e8eaed,
        context_menu_background: 0x00252830,
        context_menu_border: 0x0049505a,
        context_menu_text: 0x00e8eaed,
        context_menu_shortcut_text: 0x009098a8,
        context_menu_separator: 0x00384048,
        ..Default::default()
    }
}
