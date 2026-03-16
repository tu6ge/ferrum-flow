use gpui::*;

use crate::{
    graph::Graph,
    plugin::{
        EventResult, FlowEvent, InitPluginContext, InputEvent, Plugin, PluginContext,
        PluginRegistry, RenderContext, RenderLayer,
    },
    viewport::Viewport,
};

mod types;

pub use types::{InteractionHandler, InteractionResult, InteractionState};

pub struct FlowCanvas {
    pub graph: Graph,

    pub(crate) viewport: Viewport,

    pub(crate) plugins_registry: PluginRegistry,

    pub(crate) focus_handle: FocusHandle,

    pub(crate) interaction: InteractionState,

    pub event_queue: Vec<FlowEvent>,
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
            focus_handle,
            interaction: InteractionState::new(),
            event_queue: vec![],
        }
    }

    pub fn plugin(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugins_registry = self.plugins_registry.add(plugin);
        self
    }

    pub fn init_plugins(&mut self) {
        let mut ctx = InitPluginContext {
            graph: &mut self.graph,
            viewport: &mut self.viewport,
        };

        self.plugins_registry.plugins.sort_by_key(|p| -p.priority());

        for plugin in &mut self.plugins_registry.plugins.iter_mut() {
            plugin.setup(&mut ctx);
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
                &mut self.viewport,
                &mut self.interaction,
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
            &mut self.viewport,
            &mut self.interaction,
            &mut emit,
            &mut notify,
        );

        // 否则广播给 plugins
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
                &mut self.viewport,
                &mut self.interaction,
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
}

impl Render for FlowCanvas {
    fn render(&mut self, window: &mut Window, this_cx: &mut Context<Self>) -> impl IntoElement {
        let entity = this_cx.entity();

        let graph = &self.graph;
        let viewport = &self.viewport;

        let mut layers: Vec<Vec<AnyElement>> =
            (0..RenderLayer::ALL.len()).map(|_| Vec::new()).collect();

        for plugin in self.plugins_registry.plugins.iter_mut() {
            let layer = plugin.render_layer();

            let mut ctx = RenderContext::new(graph, viewport, window, layer);

            if let Some(el) = plugin.render(&mut ctx) {
                layers[layer.index()].push(el);
            }
        }

        if let Some(i) = self.interaction.handler.as_ref() {
            let mut ctx = RenderContext::new(graph, viewport, window, RenderLayer::Interaction);

            if let Some(el) = i.render(&mut ctx) {
                layers[RenderLayer::Interaction.index()].push(el);
            }
        }

        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(window.listener_for(&entity, Self::on_key_down))
            .on_key_up(window.listener_for(&entity, Self::on_key_up))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&entity, Self::on_mouse_down),
            )
            .on_mouse_move(window.listener_for(&entity, Self::on_mouse_move))
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&entity, Self::on_mouse_up),
            )
            .on_scroll_wheel(window.listener_for(&entity, Self::on_scroll_wheel))
            .children(
                RenderLayer::ALL
                    .iter()
                    .map(|layer| div().absolute().children(layers[layer.index()].drain(..))),
            )
    }
}
