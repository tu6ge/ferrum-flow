use std::sync::{Arc, Mutex};
use std::time::Duration;

use ferrum_flow::{
    EventResult, FlowEvent, Graph, InitPluginContext, NodeId, Plugin, PluginContext, RenderContext,
    RenderLayer,
};
use futures::{StreamExt, channel::mpsc};
use gpui::{AnyElement, Element, Point, Size, Styled, div, px, rgb};

use crate::{ExecutorContext, GraphExecutor};

const DEFAULT_STEP_DELAY: Duration = Duration::from_millis(60);

/// Highlights the node currently being executed by [`GraphExecutor`]. Send `()` on
/// [`Self::trigger`], or emit [`ExecuteGraphEvent`] via [`PluginContext::emit`], to run the
/// current editor graph step-by-step with a visible highlight.
pub struct ExecutionHighlightPlugin {
    executor: Arc<GraphExecutor>,
    current_node: Arc<Mutex<Option<NodeId>>>,
    trigger_tx: mpsc::UnboundedSender<()>,
    trigger_rx: Option<mpsc::UnboundedReceiver<()>>,
    step_delay: Duration,
    on_run_complete: Arc<Mutex<Option<Box<dyn FnMut(&mut Graph, &ExecutorContext) + Send>>>>,
}

/// Emit with [`PluginContext::emit`] to start the same run as the trigger channel.
#[derive(Clone, Copy, Debug)]
pub struct ExecuteGraphEvent;

impl ExecutionHighlightPlugin {
    pub fn new(executor: GraphExecutor) -> Self {
        let (tx, rx) = mpsc::unbounded();
        Self {
            executor: Arc::new(executor),
            current_node: Arc::new(Mutex::new(None)),
            trigger_tx: tx,
            trigger_rx: Some(rx),
            step_delay: DEFAULT_STEP_DELAY,
            on_run_complete: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_step_delay(mut self, delay: Duration) -> Self {
        self.step_delay = delay;
        self
    }

    /// Called on the main thread after a **successful** full run (all nodes executed), so you can
    /// write execution results back into the live [`Graph`] (e.g. update [`Node::data`]).
    pub fn with_on_run_complete<F>(self, f: F) -> Self
    where
        F: FnMut(&mut Graph, &ExecutorContext) + Send + 'static,
    {
        *self.on_run_complete.lock().expect("on_run_complete lock") = Some(Box::new(f));
        self
    }

    /// Clone and send `()` to schedule a graph run (snapshot of the current canvas graph).
    pub fn trigger(&self) -> mpsc::UnboundedSender<()> {
        self.trigger_tx.clone()
    }
}

impl Plugin for ExecutionHighlightPlugin {
    fn name(&self) -> &'static str {
        "execution_highlight"
    }

    fn setup(&mut self, init: &mut InitPluginContext<'_, '_>) {
        let Some(mut rx) = self.trigger_rx.take() else {
            return;
        };

        let executor = self.executor.clone();
        let current = self.current_node.clone();
        let step_delay = self.step_delay;
        let on_run_complete = self.on_run_complete.clone();

        init
            .gpui_ctx
            .spawn(async move |this, cx| {
                while let Some(()) = rx.next().await {
                    let graph = match this.update(cx, |canvas, _| canvas.graph.clone()) {
                        Ok(g) => g,
                        Err(_) => continue,
                    };

                    let (order, edge_map) = match executor.execution_plan(&graph) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    let mut exec_ctx = ExecutorContext::default();
                    let mut run_ok = true;

                    for node_id in &order {
                        if let Ok(mut g) = current.lock() {
                            *g = Some(*node_id);
                        }
                        let _ = this.update(cx, |_, c| c.notify());
                        futures_timer::Delay::new(step_delay).await;

                        if executor
                            .execute_node(&graph, node_id, &mut exec_ctx, &edge_map)
                            .is_err()
                        {
                            run_ok = false;
                            break;
                        }
                    }

                    if let Ok(mut g) = current.lock() {
                        *g = None;
                    }
                    let _ = this.update(cx, |canvas, c| {
                        if run_ok {
                            if let Ok(mut lock) = on_run_complete.lock() {
                                if let Some(f) = lock.as_mut() {
                                    f(&mut canvas.graph, &exec_ctx);
                                }
                            }
                        }
                        c.notify();
                    });
                }
            })
            .detach();
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        if event.as_custom::<ExecuteGraphEvent>().is_some() {
            let _ = self.trigger_tx.unbounded_send(());
            ctx.notify();
            return EventResult::Stop;
        }
        EventResult::Continue
    }

    fn priority(&self) -> i32 {
        65
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Interaction
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<AnyElement> {
        let id = self.current_node.lock().ok()?.as_ref().copied()?;
        if !ctx.is_node_visible(&id) {
            return None;
        }
        let node = ctx.graph.nodes().get(&id)?;
        let top_left = ctx.world_to_screen(Point::new(node.x, node.y));
        let size = Size::new(
            ctx.viewport.world_length_to_screen(node.size.width),
            ctx.viewport.world_length_to_screen(node.size.height),
        );

        Some(
            div()
                .absolute()
                .left(top_left.x)
                .top(top_left.y)
                .w(size.width)
                .h(size.height)
                .border(px(3.0))
                .border_color(rgb(0xFFAA00))
                .into_any(),
        )
    }
}
