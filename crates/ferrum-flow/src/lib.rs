pub use ferrum_flow_core::*;

pub use ferrum_flow_plugins::*;

pub trait DefaultPlugins: private::Sealed {
    fn default_plugins(self) -> Self;
}

impl DefaultPlugins for FlowCanvasBuilder<'_, '_> {
    /// Registers the **core** plugin set for editing a node graph on the canvas: background,
    /// selection, node drag, pan/zoom, node/edge rendering, port wiring, delete, and undo/redo
    /// ([`BackgroundPlugin`], [`SelectionPlugin`], [`NodeInteractionPlugin`], [`ViewportPlugin`],
    /// [`NodePlugin`], [`PortInteractionPlugin`], [`EdgePlugin`], [`DeletePlugin`], [`HistoryPlugin`]).
    ///
    /// Event order is determined by each plugin’s [`Plugin::priority`] when [`FlowCanvas::build`]
    /// runs (not by the order of calls to [`.plugin`](Self::plugin)). Add minimap, clipboard,
    /// context menu, etc. with [`.plugin`](Self::plugin) before or after this call.
    fn default_plugins(self) -> Self {
        self.plugin(BackgroundPlugin::new())
            .plugin(SelectionPlugin::new())
            .plugin(NestedNodeDragPlugin::new())
            .plugin(ViewportPlugin::new())
            .plugin(GraphPlugin::new())
            .plugin(PortInteractionPlugin::new())
            .plugin(DeletePlugin::default())
            .plugin(HistoryPlugin::new())
            .plugin(ToastPlugin::new())
    }
}

mod private {
    use super::FlowCanvasBuilder;
    pub trait Sealed {}

    impl Sealed for FlowCanvasBuilder<'_, '_> {}
}
