//! Window root: layers UI above [`ferrum_flow::FlowCanvas`].
//!
//! - [`gpui_component::select::Select`] for node type after a dangling link (see [`crate::plugins::NodeTypePickerPlugin`]).
//! - [`gpui_component::input::Input`] plus **Select** for “Add node” from the context menu: pick type and title
//!   (see [`crate::add_node_dialog`] and [`crate::plugins::MeiliAddNodePlugin`]).
//!
//! User actions forward custom events through [`ferrum_flow::FlowCanvas::handle_event`]; plugins update the graph.

use ferrum_flow::FlowCanvas;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, px, rgba,
};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState};
use gpui_component::{Root, Sizable as _, h_flex, v_flex};

use crate::add_node_dialog;
use crate::pick_state;
use crate::plugins::pick_link_event::{AddNodeConfirm, NodeTypeSelectConfirm};

#[derive(Clone)]
struct NodePickItem {
    title: SharedString,
    value: u8,
}

impl SelectItem for NodePickItem {
    type Value = u8;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &u8 {
        &self.value
    }
}

fn node_pick_items() -> SearchableVec<NodePickItem> {
    SearchableVec::new([
        NodePickItem {
            title: "Agent — orchestration · tools · memory".into(),
            value: 1,
        },
        NodePickItem {
            title: "LLM — model inference".into(),
            value: 2,
        },
        NodePickItem {
            title: "Tool — search / code / RAG".into(),
            value: 3,
        },
        NodePickItem {
            title: "Router — branching · retry".into(),
            value: 4,
        },
        NodePickItem {
            title: "Start — io_start".into(),
            value: 5,
        },
        NodePickItem {
            title: "End — io_end".into(),
            value: 6,
        },
        NodePickItem {
            title: "Step — generic".into(),
            value: 7,
        },
    ])
}

fn read_add_node_kind_digit_ctx(
    kind_select: &Entity<SelectState<SearchableVec<NodePickItem>>>,
    cx: &mut Context<MeiliShell>,
) -> u8 {
    kind_select
        .read_with(cx, |s, _| s.selected_value().copied())
        .unwrap_or(7)
}

fn read_add_node_kind_digit_app(
    kind_select: &Entity<SelectState<SearchableVec<NodePickItem>>>,
    cx: &mut App,
) -> u8 {
    kind_select
        .read_with(cx, |s, _| s.selected_value().copied())
        .unwrap_or(7)
}

fn dispatch_add_node_confirmed<C: gpui::AppContext>(
    canvas: &Entity<FlowCanvas>,
    label: SharedString,
    kind_digit: u8,
    cx: &mut C,
) {
    let t = label.trim();
    if t.is_empty() {
        return;
    }
    let label_confirmed: SharedString = t.to_string().into();
    let (world_x, world_y) = crate::add_node_dialog::take_pending_world().unwrap_or((240.0, 200.0));
    canvas.update(cx, |flow, cx| {
        flow.handle_event(
            ferrum_flow::FlowEvent::custom(AddNodeConfirm {
                label: label_confirmed,
                world_x,
                world_y,
                kind_digit,
            }),
            cx,
        );
    });
    add_node_dialog::close();
    canvas.update(cx, |_, cx| cx.notify());
}

fn flush_add_node_shell_ctx(
    canvas: &Entity<FlowCanvas>,
    input: &Entity<InputState>,
    kind_select: &Entity<SelectState<SearchableVec<NodePickItem>>>,
    cx: &mut Context<MeiliShell>,
) {
    let label: SharedString = input.read_with(cx, |i, _| i.value());
    let kind_digit = read_add_node_kind_digit_ctx(kind_select, cx);
    dispatch_add_node_confirmed(canvas, label, kind_digit, cx);
}

fn flush_add_node_app(
    canvas: &Entity<FlowCanvas>,
    input: &Entity<InputState>,
    kind_select: &Entity<SelectState<SearchableVec<NodePickItem>>>,
    cx: &mut App,
) {
    let label: SharedString = input.read_with(cx, |i, _| i.value());
    let kind_digit = read_add_node_kind_digit_app(kind_select, cx);
    dispatch_add_node_confirmed(canvas, label, kind_digit, cx);
}

fn cancel_add_node_app(canvas: &Entity<FlowCanvas>, cx: &mut App) {
    add_node_dialog::close();
    canvas.update(cx, |_, cx| cx.notify());
}

pub struct MeiliShell {
    pub canvas: Entity<FlowCanvas>,
    node_select: Entity<SelectState<SearchableVec<NodePickItem>>>,
    /// Type picker inside the add-node dialog (same item set as [`Self::node_select`]).
    add_node_kind: Entity<SelectState<SearchableVec<NodePickItem>>>,
    add_node_label: Entity<InputState>,
    _canvas_obs: Subscription,
    _select_sub: Subscription,
    _add_node_enter_sub: Subscription,
}

impl MeiliShell {
    pub fn new(canvas: Entity<FlowCanvas>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let node_select = cx.new(|cx| SelectState::new(node_pick_items(), None, window, cx));

        let add_node_label =
            cx.new(|cx| InputState::new(window, cx).placeholder("Node display name…"));

        let add_node_kind = cx.new(|cx| {
            let mut state = SelectState::new(node_pick_items(), None, window, cx).searchable(true);
            state.set_selected_value(&7u8, window, cx);
            state
        });

        let canvas_obs = cx.observe(&canvas, |_shell, _, cx| {
            cx.notify();
        });

        let canvas_for_confirm = canvas.clone();
        let select_sub = cx.subscribe(
            &node_select,
            move |_shell, _, event: &SelectEvent<SearchableVec<NodePickItem>>, cx| {
                if let SelectEvent::Confirm(Some(digit)) = event {
                    canvas_for_confirm.update(cx, |flow, cx| {
                        flow.handle_event(
                            ferrum_flow::FlowEvent::custom(NodeTypeSelectConfirm { digit: *digit }),
                            cx,
                        );
                    });
                }
            },
        );

        let canvas_enter = canvas.clone();
        let input_enter = add_node_label.clone();
        let kind_enter = add_node_kind.clone();
        let add_node_enter_sub =
            cx.subscribe(&add_node_label, move |_shell, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    flush_add_node_shell_ctx(&canvas_enter, &input_enter, &kind_enter, cx);
                }
            });

        Self {
            canvas,
            node_select,
            add_node_kind,
            add_node_label,
            _canvas_obs: canvas_obs,
            _select_sub: select_sub,
            _add_node_enter_sub: add_node_enter_sub,
        }
    }
}

impl Render for MeiliShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if add_node_dialog::is_open() && add_node_dialog::take_should_clear_input() {
            let _ = self.add_node_label.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            let _ = self.add_node_kind.update(cx, |state, cx| {
                state.set_selected_value(&7u8, window, cx);
            });
        }

        let show_bar = pick_state::pending_peek().is_some();
        let show_add_node = add_node_dialog::is_open();

        let canvas_ok = self.canvas.clone();
        let input_ok = self.add_node_label.clone();
        let kind_ok = self.add_node_kind.clone();
        let canvas_cancel = self.canvas.clone();

        div()
            .size_full()
            .relative()
            .child(self.canvas.clone())
            .when(show_bar, |this| {
                this.child(
                    div()
                        .absolute()
                        .bottom(px(24.0))
                        .left(px(0.0))
                        .right(px(0.0))
                        .flex()
                        .justify_center()
                        .child(
                            div().w(px(380.0)).child(
                                Select::new(&self.node_select)
                                    .placeholder("Choose node type")
                                    .menu_width(px(380.0))
                                    .small(),
                            ),
                        ),
                )
            })
            .when(show_add_node, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(px(0.0))
                        .left(px(0.0))
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba(0x000000aa))
                        .child(
                            v_flex()
                                .w(px(400.0))
                                .p(px(20.0))
                                .rounded(px(10.0))
                                .bg(rgba(0x121824f0))
                                .border_1()
                                .border_color(rgba(0xffffff18))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgba(0xe8ecf1))
                                        .child("Add node"),
                                )
                                .child(
                                    div()
                                        .mt_2()
                                        .text_xs()
                                        .text_color(rgba(0x8b98a8))
                                        .child("The card is centered on the canvas point where you right-clicked; the title is yours to set."),
                                )
                                .child(
                                    div()
                                        .mt_3()
                                        .text_xs()
                                        .text_color(rgba(0x8b98a8))
                                        .child("Node type"),
                                )
                                .child(
                                    div().mt_1().child(
                                        Select::new(&self.add_node_kind)
                                            .placeholder("Choose type")
                                            .menu_width(px(360.0))
                                            .small(),
                                    ),
                                )
                                .child(
                                    div()
                                        .mt_3()
                                        .text_xs()
                                        .text_color(rgba(0x8b98a8))
                                        .child("Display name"),
                                )
                                .child(
                                    div().mt_1().child(
                                        Input::new(&self.add_node_label)
                                            .small()
                                            .cleanable(true),
                                    ),
                                )
                                .child(
                                    h_flex()
                                        .mt_4()
                                        .justify_end()
                                        .child(
                                            Button::new("cancel-add-node")
                                                .small()
                                                .ghost()
                                                .on_click({
                                                    let c = canvas_cancel.clone();
                                                    move |_, _, cx: &mut App| {
                                                        cancel_add_node_app(&c, cx);
                                                    }
                                                })
                                                .child("Cancel"),
                                        )
                                        .child(
                                            Button::new("ok-add-node")
                                                .small()
                                                .primary()
                                                .ml_2()
                                                .on_click({
                                                    let c = canvas_ok.clone();
                                                    let inp = input_ok.clone();
                                                    let k = kind_ok.clone();
                                                    move |_, _, cx: &mut App| {
                                                        flush_add_node_app(&c, &inp, &k, cx);
                                                    }
                                                })
                                                .child("Add"),
                                        ),
                                ),
                        ),
                )
            })
    }
}

/// Built from [`crate::main`]: `Root` is the window root (required by gpui-component); [`MeiliShell`] is inside.
pub fn window_root(
    canvas: Entity<FlowCanvas>,
    window: &mut Window,
    cx: &mut Context<Root>,
) -> Root {
    let shell = cx.new(|ctx| MeiliShell::new(canvas, window, ctx));
    Root::new(shell, window, cx)
}
