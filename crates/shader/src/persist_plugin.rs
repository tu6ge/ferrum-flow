//! Shortcuts: save / open; top-center toasts (decoupled from stderr).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ferrum_flow::{
    EventResult, FlowEvent, InitPluginContext, InputEvent, Plugin, PluginContext, RenderContext,
    RenderLayer, primary_platform_modifier,
};
use gpui::{
    AsyncApp, FontWeight, IntoElement as _, ParentElement as _, Styled as _, Task, WeakEntity, div,
    px, rgb, rgba,
};

use crate::graph_io::{load_graph_from_path, replace_canvas_graph, save_graph_to_path};
use crate::validate::graph_validation_notes;

const TOAST_DURATION: Duration = Duration::from_secs(4);
const TOAST_TICK: Duration = Duration::from_millis(200);
const DETAIL_LINES_MAX: usize = 10;

#[derive(Clone, Copy)]
enum ToastKind {
    Success,
    Error,
}

#[derive(Clone)]
struct ToastState {
    kind: ToastKind,
    title: String,
    detail: Vec<String>,
    until: Instant,
}

pub struct ShaderGraphFilePlugin {
    path: PathBuf,
    toast: Arc<Mutex<Option<ToastState>>>,
    /// Keeps the async tick alive so expired toasts still trigger a redraw without input.
    _toast_tick: Option<Task<()>>,
}

impl ShaderGraphFilePlugin {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            toast: Arc::new(Mutex::new(None)),
            _toast_tick: None,
        }
    }

    fn push_toast(
        toast: &Arc<Mutex<Option<ToastState>>>,
        ctx: &mut PluginContext,
        kind: ToastKind,
        title: impl Into<String>,
        detail: Vec<String>,
    ) {
        let detail: Vec<String> = detail.into_iter().take(DETAIL_LINES_MAX).collect();
        let mut g = toast.lock().unwrap_or_else(|e| e.into_inner());
        *g = Some(ToastState {
            kind,
            title: title.into(),
            detail,
            until: Instant::now() + TOAST_DURATION,
        });
        ctx.notify();
    }

    fn validation_detail(graph: &ferrum_flow::Graph) -> Vec<String> {
        graph_validation_notes(graph)
    }
}

impl Plugin for ShaderGraphFilePlugin {
    fn name(&self) -> &'static str {
        "shader_graph_file"
    }

    fn setup(&mut self, ctx: &mut InitPluginContext) {
        let toast = Arc::clone(&self.toast);
        self._toast_tick = Some(ctx.gpui_ctx.spawn(
            async move |weak: WeakEntity<ferrum_flow::FlowCanvas>, cx: &mut AsyncApp| {
                loop {
                    cx.background_executor().timer(TOAST_TICK).await;
                    let should_notify = {
                        let mut g = toast.lock().unwrap_or_else(|e| e.into_inner());
                        match g.as_ref() {
                            Some(t) if Instant::now() > t.until => {
                                *g = None;
                                true
                            }
                            _ => false,
                        }
                    };
                    if should_notify {
                        let _ = weak.update(cx, |_, view_cx| {
                            view_cx.notify();
                        });
                    }
                }
            },
        ));
    }

    /// Below minimap (135), above context menu (132): toasts under chrome, menus on top.
    fn priority(&self) -> i32 {
        134
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    fn render(&mut self, ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let toast = self
            .toast
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()?;

        if Instant::now() > toast.until {
            return None;
        }

        let (accent, border) = match toast.kind {
            ToastKind::Success => (0x004ade80_u32, 0x004ade80_u32),
            ToastKind::Error => (0x00ff6b6b_u32, 0x00ff6b6b_u32),
        };

        let fg = ctx.theme.context_menu_text;
        let muted = ctx.theme.context_menu_shortcut_text;

        let title_el = div()
            .text_sm()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(fg))
            .child(toast.title.clone());

        let detail_els = toast.detail.iter().map(|line| {
            div()
                .text_xs()
                .text_color(rgb(muted))
                .child(format!("· {line}"))
                .into_any_element()
        });

        let panel = div()
            .max_w(px(440.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(rgb(border))
            .bg(rgba(ctx.theme.context_menu_background << 8 | 0xe4))
            .shadow_sm()
            .px_4()
            .py_3()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex_shrink_0()
                            .mt(px(2.0))
                            .w(px(4.0))
                            .h(px(18.0))
                            .rounded_full()
                            .bg(rgb(accent)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .min_w(px(0.0))
                            .child(title_el)
                            .children(detail_els),
                    ),
            );

        Some(
            div()
                .absolute()
                .size_full()
                .child(
                    div()
                        .absolute()
                        .top(px(20.0))
                        .left(px(0.0))
                        .right(px(0.0))
                        .flex()
                        .justify_center()
                        .child(panel),
                )
                .into_any_element(),
        )
    }

    fn on_event(&mut self, event: &FlowEvent, ctx: &mut PluginContext) -> EventResult {
        let FlowEvent::Input(InputEvent::KeyDown(ev)) = event else {
            return EventResult::Continue;
        };

        if !primary_platform_modifier(ev) {
            return EventResult::Continue;
        }

        if ev.keystroke.key == "s" && !ev.keystroke.modifiers.shift {
            let notes = Self::validation_detail(ctx.graph);
            match save_graph_to_path(ctx.graph, &self.path) {
                Ok(()) => {
                    let mut detail = vec![format!("{}", self.path.display())];
                    detail.extend(notes);
                    Self::push_toast(&self.toast, ctx, ToastKind::Success, "Saved", detail);
                }
                Err(e) => {
                    Self::push_toast(&self.toast, ctx, ToastKind::Error, "Save failed", vec![e]);
                }
            }
            ctx.notify();
            return EventResult::Stop;
        }

        if ev.keystroke.key == "o" && !ev.keystroke.modifiers.shift {
            match load_graph_from_path(&self.path) {
                Ok(g) => {
                    let notes = Self::validation_detail(&g);
                    replace_canvas_graph(ctx, g);
                    let mut detail = vec![format!("{}", self.path.display())];
                    detail.extend(notes);
                    Self::push_toast(&self.toast, ctx, ToastKind::Success, "Opened", detail);
                }
                Err(e) => {
                    Self::push_toast(&self.toast, ctx, ToastKind::Error, "Open failed", vec![e]);
                }
            }
            ctx.notify();
            return EventResult::Stop;
        }

        EventResult::Continue
    }
}
