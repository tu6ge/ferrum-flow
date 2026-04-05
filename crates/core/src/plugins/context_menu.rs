use gpui::{
    IntoElement as _, MouseButton, ParentElement as _, Point, Pixels, Styled as _, div, px, rgb,
};

use crate::{
    NodeId,
    plugin::{
        EventResult, FlowEvent, InputEvent, Plugin, PluginContext, RenderContext, RenderLayer,
    },
};

use super::{
    clipboard_ops::{extract_subgraph, paste_subgraph},
    delete::delete_selection,
    focus_selection::focus_viewport_on_selection,
    select_all_viewport::select_all_in_viewport,
};

const MENU_W: f32 = 228.0;
const ROW_H: f32 = 26.0;
const SEP_H: f32 = 9.0;
const MENU_PAD: f32 = 4.0;

/// Right-click menu on the canvas (empty area) or on a node. Thin wrapper over existing graph actions.
pub struct ContextMenuPlugin {
    open: Option<OpenMenu>,
}

#[derive(Clone)]
struct OpenMenu {
    anchor: Point<Pixels>,
    actions: Vec<MenuAction>,
}

#[derive(Clone)]
enum MenuAction {
    Paste,
    SelectAllViewport,
    FocusSelection,
    Copy,
    Delete,
    BringToFront(NodeId),
    Separator,
}

impl ContextMenuPlugin {
    pub fn new() -> Self {
        Self { open: None }
    }

    fn row_height(action: &MenuAction) -> f32 {
        match action {
            MenuAction::Separator => SEP_H,
            _ => ROW_H,
        }
    }

    fn content_height(actions: &[MenuAction]) -> f32 {
        actions.iter().map(Self::row_height).sum()
    }

    fn menu_bounds(anchor: Point<Pixels>, actions: &[MenuAction]) -> gpui::Bounds<Pixels> {
        let h = Self::content_height(actions) + MENU_PAD * 2.0;
        gpui::Bounds::new(anchor, gpui::Size::new(px(MENU_W), px(h)))
    }

    fn label(action: &MenuAction) -> &'static str {
        match action {
            MenuAction::Paste => "Paste",
            MenuAction::SelectAllViewport => "Select all in view",
            MenuAction::FocusSelection => "Focus selection",
            MenuAction::Copy => "Copy",
            MenuAction::Delete => "Delete",
            MenuAction::BringToFront(_) => "Bring to front",
            MenuAction::Separator => "",
        }
    }

    /// Matches [`ClipboardPlugin`], [`SelectAllViewportPlugin`], [`FocusSelectionPlugin`], [`DeletePlugin`].
    fn shortcut_hint(action: &MenuAction) -> Option<&'static str> {
        #[cfg(target_os = "macos")]
        {
            match action {
                MenuAction::Paste => Some("⌘V"),
                MenuAction::SelectAllViewport => Some("⌘A"),
                MenuAction::FocusSelection => Some("⌘⇧F"),
                MenuAction::Copy => Some("⌘C"),
                MenuAction::Delete => Some("⌫"),
                MenuAction::BringToFront(_) => None,
                MenuAction::Separator => None,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            match action {
                MenuAction::Paste => Some("Ctrl+V"),
                MenuAction::SelectAllViewport => Some("Ctrl+A"),
                MenuAction::FocusSelection => Some("Ctrl+Shift+F"),
                MenuAction::Copy => Some("Ctrl+C"),
                MenuAction::Delete => Some("Del"),
                MenuAction::BringToFront(_) => None,
                MenuAction::Separator => None,
            }
        }
    }

    fn canvas_actions(ctx: &PluginContext) -> Vec<MenuAction> {
        let mut v = Vec::new();
        if ctx.clipboard_subgraph.is_some() {
            v.push(MenuAction::Paste);
            v.push(MenuAction::Separator);
        }
        v.push(MenuAction::SelectAllViewport);
        v.push(MenuAction::Separator);
        v.push(MenuAction::FocusSelection);
        v
    }

    fn node_actions(nid: NodeId) -> Vec<MenuAction> {
        vec![
            MenuAction::Copy,
            MenuAction::Separator,
            MenuAction::Delete,
            MenuAction::BringToFront(nid),
            MenuAction::Separator,
            MenuAction::FocusSelection,
        ]
    }

    fn run_action(ctx: &mut PluginContext, action: &MenuAction) {
        match action {
            MenuAction::Separator => {}
            MenuAction::Paste => {
                if let Some(sub) = ctx.clipboard_subgraph.clone() {
                    paste_subgraph(ctx, &sub);
                }
            }
            MenuAction::SelectAllViewport => select_all_in_viewport(ctx),
            MenuAction::FocusSelection => focus_viewport_on_selection(ctx),
            MenuAction::Copy => {
                if let Some(s) = extract_subgraph(ctx.graph) {
                    *ctx.clipboard_subgraph = Some(s);
                }
            }
            MenuAction::Delete => delete_selection(ctx),
            MenuAction::BringToFront(id) => ctx.bring_node_to_front(*id),
        }
        ctx.notify();
    }

    /// Row index under `dy` (pixels from top of inner content, below top padding).
    fn row_at_dy(actions: &[MenuAction], dy: f32) -> Option<usize> {
        if dy < 0.0 {
            return None;
        }
        let mut y = 0.0;
        for (i, a) in actions.iter().enumerate() {
            let h = Self::row_height(a);
            if dy < y + h {
                return Some(i);
            }
            y += h;
        }
        None
    }
}

impl Plugin for ContextMenuPlugin {
    fn name(&self) -> &'static str {
        "context_menu"
    }

    fn setup(&mut self, _ctx: &mut crate::plugin::InitPluginContext) {}

    fn priority(&self) -> i32 {
        132
    }

    fn render_layer(&self) -> RenderLayer {
        RenderLayer::Overlay
    }

    fn render(&mut self, _ctx: &mut RenderContext) -> Option<gpui::AnyElement> {
        let open = self.open.as_ref()?;
        let rows: Vec<_> = open
            .actions
            .iter()
            .map(|a| match a {
                MenuAction::Separator => div()
                    .w_full()
                    .h(px(SEP_H))
                    .flex()
                    .items_center()
                    .px_2()
                    .child(
                        div()
                            .w_full()
                            .h(px(1.0))
                            .bg(rgb(0xe0e0e8)),
                    )
                    .into_any_element(),
                _ => {
                    let label = div()
                        .flex_1()
                        .min_w(px(0.))
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(Self::label(a));
                    let shortcut = Self::shortcut_hint(a).map(|h| {
                        div()
                            .flex_shrink_0()
                            .ml_2()
                            .text_xs()
                            .text_color(rgb(0x7a7a88))
                            .child(h)
                    });
                    div()
                        .w_full()
                        .h(px(ROW_H))
                        .flex()
                        .flex_row()
                        .items_center()
                        .px_2()
                        .text_sm()
                        .text_color(rgb(0x1a192b))
                        .child(label)
                        .children(shortcut)
                        .into_any_element()
                }
            })
            .collect();

        Some(
            div()
                .absolute()
                .left(open.anchor.x)
                .top(open.anchor.y)
                .w(px(MENU_W))
                .p_1()
                .bg(rgb(0xfcfcfc))
                .border_1()
                .border_color(rgb(0xc8c8d0))
                .rounded(px(6.0))
                .shadow_sm()
                .children(rows)
                .into_any_element(),
        )
    }

    fn on_event(
        &mut self,
        event: &FlowEvent,
        ctx: &mut PluginContext,
    ) -> crate::plugin::EventResult {
        if let FlowEvent::Input(InputEvent::MouseDown(ev)) = event {
            if ev.button == MouseButton::Left {
                if let Some(open) = self.open.take() {
                    let b = Self::menu_bounds(open.anchor, &open.actions);
                    if b.contains(&ev.position) {
                        let dy: f32 = (ev.position.y - open.anchor.y).into();
                        let inner_y = dy - MENU_PAD;
                        if let Some(row) = Self::row_at_dy(&open.actions, inner_y) {
                            let a = &open.actions[row];
                            if !matches!(a, MenuAction::Separator) {
                                Self::run_action(ctx, a);
                            } else {
                                ctx.notify();
                            }
                        } else {
                            ctx.notify();
                        }
                        return EventResult::Stop;
                    }
                    ctx.notify();
                    return EventResult::Continue;
                }
                return EventResult::Continue;
            }

            if ev.button == MouseButton::Right {
                let world = ctx.screen_to_world(ev.position);
                let actions = if let Some(nid) = ctx.hit_node(world) {
                    if !ctx.graph.selected_node.contains(&nid) {
                        ctx.clear_selected_edge();
                        ctx.clear_selected_node();
                        ctx.add_selected_node(nid, false);
                    }
                    Self::node_actions(nid)
                } else {
                    Self::canvas_actions(ctx)
                };
                self.open = Some(OpenMenu {
                    anchor: ev.position,
                    actions,
                });
                ctx.notify();
                return EventResult::Stop;
            }
        }
        EventResult::Continue
    }
}
