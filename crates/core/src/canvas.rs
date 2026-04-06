use futures::{StreamExt, channel::mpsc};
use gpui::*;

use crate::{
    GraphChange, SyncPlugin,
    copied_subgraph::CopiedSubgraph,
    graph::Graph,
    plugin::{
        EventResult, FlowEvent, InitPluginContext, InputEvent, Plugin, PluginContext,
        PluginRegistry, RenderContext, RenderLayer,
    },
    viewport::Viewport,
};

mod node_renderer;
mod port_cache;
mod types;
mod undo;

pub use port_cache::PortLayoutCache;

pub use undo::{Command, CommandContext, CompositeCommand, HistoryProvider, LocalHistory};

pub use types::{Interaction, InteractionResult, InteractionState};

pub use node_renderer::{NodeRenderer, RendererRegistry, port_screen_position};

pub struct FlowCanvas {
    pub graph: Graph,

    pub(crate) viewport: Viewport,

    pub(crate) plugins_registry: PluginRegistry,

    pub(crate) sync_plugin: Option<Box<dyn SyncPlugin + 'static>>,

    renderers: RendererRegistry,

    pub(crate) focus_handle: FocusHandle,

    pub(crate) interaction: InteractionState,

    pub history: Box<dyn HistoryProvider>,

    pub event_queue: Vec<FlowEvent>,
    pub port_offset_cache: PortLayoutCache,

    /// Shared with [`crate::plugins::ClipboardPlugin`] and context menu.
    pub(crate) clipboard_subgraph: Option<CopiedSubgraph>,
}

// // TODO
// impl Clone for FlowCanvas {
//     fn clone(&self) -> Self {
//         Self {
//             graph: self.graph.clone(),
//             viewport: self.viewport.clone(),
//             plugins_registry: PluginRegistry::new(),
//             focus_handle: self.focus_handle.clone(),
//             interaction: InteractionState::new(),
//             event_queue: vec![],
//         }
//     }
// }

impl FlowCanvas {
    pub fn new(graph: Graph, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            graph,
            viewport: Viewport::new(),
            plugins_registry: PluginRegistry::new(),
            sync_plugin: None,
            renderers: RendererRegistry::new(),
            focus_handle,
            interaction: InteractionState::new(),
            history: Box::new(LocalHistory::new()),
            event_queue: vec![],
            port_offset_cache: PortLayoutCache::new(),
            clipboard_subgraph: None,
        }
    }

    pub fn builder<'a, 'b>(
        graph: Graph,
        ctx: &'a mut Context<'b, Self>,
    ) -> FlowCanvasBuilder<'a, 'b> {
        FlowCanvasBuilder {
            graph,
            ctx,
            plugins: PluginRegistry::new(),
            sync_plugin: None,
            renderers: RendererRegistry::new(),
        }
    }

    pub fn handle_event(&mut self, event: FlowEvent, cx: &mut Context<Self>) {
        let event_queue = &mut self.event_queue;

        let mut emit = |event: FlowEvent| {
            event_queue.push(event);
        };

        let mut notify = || {
            cx.notify();
        };

        // if has interaction
        if let Some(mut handler) = self.interaction.handler.take() {
            let mut ctx = PluginContext::new(
                &mut self.graph,
                &mut self.port_offset_cache,
                &mut self.viewport,
                &mut self.interaction,
                &mut self.renderers,
                &mut self.sync_plugin,
                self.history.as_mut(),
                &mut self.clipboard_subgraph,
                &mut emit,
                &mut notify,
            );
            let mut fast_return = false;
            let result = match &event {
                FlowEvent::Input(InputEvent::MouseMove(ev)) => {
                    fast_return = true;
                    handler.on_mouse_move(ev, &mut ctx)
                }

                FlowEvent::Input(InputEvent::MouseUp(ev)) => {
                    fast_return = true;
                    handler.on_mouse_up(ev, &mut ctx)
                }

                _ => InteractionResult::Continue,
            };

            if fast_return {
                match result {
                    InteractionResult::Continue => self.interaction.handler = Some(handler),

                    InteractionResult::End => {
                        self.interaction.handler = None;
                    }

                    InteractionResult::Replace(h) => {
                        self.interaction.handler = Some(h);
                    }
                }
                return;
            }
        }

        let mut ctx = PluginContext::new(
            &mut self.graph,
            &mut self.port_offset_cache,
            &mut self.viewport,
            &mut self.interaction,
            &mut self.renderers,
            &mut self.sync_plugin,
            self.history.as_mut(),
            &mut self.clipboard_subgraph,
            &mut emit,
            &mut notify,
        );

        for plugin in &mut self.plugins_registry.plugins {
            let result = plugin.on_event(&event, &mut ctx);
            match result {
                EventResult::Continue => {}
                EventResult::Stop => break,
            }
        }
    }

    fn process_event_queue(&mut self, cx: &mut Context<Self>) {
        while let Some(event) = self.event_queue.pop() {
            let mut emit = |e| self.event_queue.push(e);

            let mut notify = || {
                cx.notify();
            };

            let mut ctx = PluginContext::new(
                &mut self.graph,
                &mut self.port_offset_cache,
                &mut self.viewport,
                &mut self.interaction,
                &mut self.renderers,
                &mut self.sync_plugin,
                self.history.as_mut(),
                &mut self.clipboard_subgraph,
                &mut emit,
                &mut notify,
            );

            for plugin in &mut self.plugins_registry.plugins {
                let result = plugin.on_event(&event, &mut ctx);
                match result {
                    EventResult::Continue => {}
                    EventResult::Stop => break,
                }
            }
        }
    }

    fn on_key_down(&mut self, ev: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::KeyDown(ev.clone())), cx);
        self.process_event_queue(cx);
    }

    fn on_key_up(&mut self, ev: &KeyUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::KeyUp(ev.clone())), cx);
        self.process_event_queue(cx);
    }

    fn on_mouse_down(&mut self, ev: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::MouseDown(ev.clone())), cx);
        self.process_event_queue(cx);
    }

    fn on_mouse_move(&mut self, ev: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::MouseMove(ev.clone())), cx);
        if let Some(sync_plugin) = &mut self.sync_plugin {
            let world = self.viewport.screen_to_world(ev.position);
            sync_plugin.on_mouse_move(ev, world);
        }
        self.process_event_queue(cx);
    }

    fn on_mouse_up(&mut self, ev: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::MouseUp(ev.clone())), cx);
        self.process_event_queue(cx);
    }

    fn on_scroll_wheel(&mut self, ev: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_event(FlowEvent::Input(InputEvent::Wheel(ev.clone())), cx);
        self.process_event_queue(cx);
    }

    fn on_canvas_hover(&mut self, hovered: &bool, _: &mut Window, cx: &mut Context<Self>) {
        if !*hovered {
            if let Some(sync_plugin) = &mut self.sync_plugin {
                sync_plugin.on_mouse_leave();
            }
            cx.notify();
        }
    }
}

impl Render for FlowCanvas {
    fn render(&mut self, window: &mut Window, this_cx: &mut Context<Self>) -> impl IntoElement {
        self.viewport.sync_drawable_bounds(window);

        let entity = this_cx.entity();

        let graph = &mut self.graph;
        let viewport = &self.viewport;
        let renderder = &self.renderers;
        let port_offset_cache = &mut self.port_offset_cache;

        let mut layers: Vec<Vec<AnyElement>> =
            (0..RenderLayer::ALL.len()).map(|_| Vec::new()).collect();

        let alignment_guides = self.interaction.alignment_guides.as_ref();

        for plugin in self.plugins_registry.plugins.iter_mut() {
            let layer = plugin.render_layer();

            let mut ctx = RenderContext::new(
                graph,
                port_offset_cache,
                viewport,
                renderder,
                window,
                layer,
                alignment_guides,
            );

            if let Some(el) = plugin.render(&mut ctx) {
                layers[layer.index()].push(el);
            }
        }

        if let Some(i) = self.interaction.handler.as_ref() {
            let mut ctx = RenderContext::new(
                graph,
                port_offset_cache,
                viewport,
                renderder,
                window,
                RenderLayer::Interaction,
                alignment_guides,
            );

            if let Some(el) = i.render(&mut ctx) {
                layers[RenderLayer::Interaction.index()].push(el);
            }
        }

        if let Some(sync_plugin) = &mut self.sync_plugin {
            let mut ctx = RenderContext::new(
                graph,
                port_offset_cache,
                viewport,
                renderder,
                window,
                RenderLayer::Overlay,
                alignment_guides,
            );
            let els = sync_plugin.render(&mut ctx);
            for el in els {
                layers[RenderLayer::Overlay.index()].push(el);
            }
        }

        div()
            .id("ferrum_flow_canvas")
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(window.listener_for(&entity, Self::on_key_down))
            .on_key_up(window.listener_for(&entity, Self::on_key_up))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&entity, Self::on_mouse_down),
            )
            .on_mouse_down(
                MouseButton::Right,
                window.listener_for(&entity, Self::on_mouse_down),
            )
            .on_mouse_move(window.listener_for(&entity, Self::on_mouse_move))
            .on_hover(window.listener_for(&entity, Self::on_canvas_hover))
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&entity, Self::on_mouse_up),
            )
            .on_scroll_wheel(window.listener_for(&entity, Self::on_scroll_wheel))
            .children(RenderLayer::ALL.iter().map(|layer| {
                div()
                    .absolute()
                    .size_full()
                    .children(layers[layer.index()].drain(..))
            }))
    }
}

pub struct FlowCanvasBuilder<'a, 'b> {
    graph: Graph,
    ctx: &'a mut Context<'b, FlowCanvas>,

    plugins: PluginRegistry,
    renderers: RendererRegistry,
    sync_plugin: Option<Box<dyn SyncPlugin + 'static>>,
}

impl<'a, 'b> FlowCanvasBuilder<'a, 'b> {
    /// register plugin
    pub fn plugin(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugins = self.plugins.add(plugin);
        self
    }

    pub fn sync_plugin(mut self, plugin: impl SyncPlugin + 'static) -> Self {
        self.sync_plugin = Some(Box::new(plugin));
        self
    }

    /// register node renderer
    pub fn node_renderer<R>(mut self, name: impl Into<String>, renderer: R) -> Self
    where
        R: node_renderer::NodeRenderer + 'static,
    {
        self.renderers.register(name, renderer);
        self
    }

    pub fn build(self) -> FlowCanvas {
        let focus_handle = self.ctx.focus_handle();

        let mut canvas = FlowCanvas {
            graph: self.graph,
            viewport: Viewport::new(),
            plugins_registry: self.plugins,
            sync_plugin: self.sync_plugin,
            renderers: self.renderers,
            focus_handle,
            interaction: InteractionState::new(),
            history: Box::new(LocalHistory::new()),
            event_queue: vec![],
            port_offset_cache: PortLayoutCache::new(),
            clipboard_subgraph: None,
        };

        if let Some(sync_plugin) = &mut canvas.sync_plugin {
            let (change_sender, mut change_receiver) = mpsc::unbounded::<GraphChange>();

            self.ctx
                .spawn(async move |this, ctx| {
                    while let Some(change) = change_receiver.next().await {
                        let _ = this.update(ctx, |this, cx| {
                            this.graph.apply(change.kind);
                            cx.notify();
                        });
                    }
                })
                .detach();
            sync_plugin.setup(change_sender);
        }

        canvas
            .plugins_registry
            .plugins
            .sort_by_key(|p| -p.priority());

        {
            let mut ctx = InitPluginContext {
                graph: &mut canvas.graph,
                port_offset_cache: &mut canvas.port_offset_cache,
                viewport: &mut canvas.viewport,
                renderers: &mut canvas.renderers,
                gpui_ctx: &self.ctx,
                //notify: &mut notify,
            };

            for plugin in &mut canvas.plugins_registry.plugins {
                plugin.setup(&mut ctx);
            }
        }

        canvas
    }
}
