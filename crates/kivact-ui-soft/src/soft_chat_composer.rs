//! Role: Composer controls for application-owned chat state.

use std::cell::RefCell;
use std::rc::Rc;

use crate::{ChatDensity, tokens};
use kivact::control::{Node, NodeBuilder, node, row, text};
use kivact::i18n::message;
use kivact::primitive::{EditState, NodeExt, NodeRefExt, part};
use kivact::style::{Align, Direction, Justify, pct, style};
use kivact::ui::{ControlSize, TextInput, text_area};
use kivact::view::{
    Handled, HandlerResult, IntoText, IntoValue, IntoView, Key, KeyEvent, Modifiers, Role, View,
    callback, node_ref, show,
};
use kivact::{Binding, Callback, Value, signal};

fn composer_placeholder() -> Value<String> {
    message("soft.chat_composer.placeholder", "Type a message…")
}

pub fn chat_composer_input(label: impl IntoText, value: impl Into<Binding<String>>) -> TextInput {
    text_area(label, value)
        .rows(1..=8)
        .placeholder(composer_placeholder())
        .style(move || {
            let p = tokens();
            style()
                .padding(4)
                .border(0, p.colors.border)
                .background(kivact::style::Color::TRANSPARENT)
                .focus_visible(style().border(0, p.colors.border))
        })
}

pub struct ChatSendButton {
    root: Node,
    stop_shown: Value<bool>,
    disabled: Value<bool>,
    on_send: Option<Callback<()>>,
    on_stop: Option<Callback<()>>,
    send_icon: Option<View>,
    stop_icon: Option<View>,
    size: ControlSize,
}

pub fn chat_send_button() -> ChatSendButton {
    ChatSendButton {
        root: node(),
        stop_shown: false.into_value(),
        disabled: false.into_value(),
        on_send: None,
        on_stop: None,
        send_icon: None,
        stop_icon: None,
        size: ControlSize::Medium,
    }
}

impl NodeBuilder for ChatSendButton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatSendButton {
    pub fn stop_shown(mut self, shown: impl IntoValue<bool>) -> Self {
        self.stop_shown = shown.into_value();
        self
    }
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn on_send<R: HandlerResult>(mut self, send: impl Fn() -> R + 'static) -> Self {
        self.on_send = Some(callback(move |_| send()));
        self
    }
    pub fn on_stop<R: HandlerResult>(mut self, stop: impl Fn() -> R + 'static) -> Self {
        self.on_stop = Some(callback(move |_| stop()));
        self
    }
    pub fn send_icon(mut self, icon: impl IntoView) -> Self {
        self.send_icon = Some(icon.into_view());
        self
    }
    pub fn stop_icon(mut self, icon: impl IntoView) -> Self {
        self.stop_icon = Some(icon.into_view());
        self
    }
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

fn control_side(size: ControlSize) -> i32 {
    match size {
        ControlSize::ExtraSmall => 24,
        ControlSize::Small => 28,
        ControlSize::Medium => 32,
        ControlSize::Large => 36,
    }
}

impl IntoView for ChatSendButton {
    fn into_view(self) -> View {
        let Self {
            root,
            stop_shown,
            disabled,
            on_send,
            on_stop,
            send_icon,
            stop_icon,
            size,
        } = self;
        let side = control_side(size);
        let send = node()
            .hidden(stop_shown)
            .child(send_icon.unwrap_or_else(|| text("↑").into_view()));
        let stop = node()
            .hidden(move || !stop_shown.get())
            .child(stop_icon.unwrap_or_else(|| text("■").into_view()));
        let send_label = message("soft.chat_send_button.send", "Send");
        let stop_label = message("soft.chat_send_button.stop", "Stop");
        root.part(
            part(|state| state)
                .role(Role::Button)
                .focusable(true)
                .disabled(disabled)
                .name(move || {
                    if stop_shown.get() {
                        stop_label.get()
                    } else {
                        send_label.get()
                    }
                })
                .on_press(move || {
                    let action = if stop_shown.get() { on_stop } else { on_send };
                    if let Some(action) = action {
                        action.call(());
                    }
                }),
        )
        .base_style(move || {
            let p = tokens();
            style()
                .direction(Direction::Row)
                .size(side, side)
                .shrink(0)
                .radius(side as f32 / 2.0)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .font_size(18)
                .background(p.colors.accent)
                .color(p.colors.on_accent)
                .hover(style().background(p.colors.accent_hover))
                .disabled(style().opacity(0.5))
                .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
        })
        .child((send, stop))
        .into_view()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComposerElevation {
    None,
    #[default]
    Low,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposerStatus {
    Error(String),
    Warning(String),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComposerStatusPosition {
    Top,
    #[default]
    Bottom,
}

pub struct ChatComposer {
    root: Node,
    value: Binding<String>,
    on_submit: Option<Callback<String>>,
    on_stop: Option<Callback<()>>,
    stop_shown: Value<bool>,
    disabled: Value<bool>,
    placeholder: Value<String>,
    density: ChatDensity,
    elevation: ComposerElevation,
    drawer: Option<View>,
    header_actions: Option<View>,
    header_context: Option<View>,
    footer_actions: Option<View>,
    send_actions: Option<View>,
    status: Value<Option<ComposerStatus>>,
    status_position: ComposerStatusPosition,
    history: bool,
}

pub fn chat_composer(value: impl Into<Binding<String>>) -> ChatComposer {
    ChatComposer {
        root: node(),
        value: value.into(),
        on_submit: None,
        on_stop: None,
        stop_shown: false.into_value(),
        disabled: false.into_value(),
        placeholder: composer_placeholder(),
        density: ChatDensity::Balanced,
        elevation: ComposerElevation::Low,
        drawer: None,
        header_actions: None,
        header_context: None,
        footer_actions: None,
        send_actions: None,
        status: None.into_value(),
        status_position: ComposerStatusPosition::Bottom,
        history: true,
    }
}

impl NodeBuilder for ChatComposer {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatComposer {
    pub fn on_submit<R: HandlerResult>(mut self, submit: impl Fn(String) -> R + 'static) -> Self {
        self.on_submit = Some(callback(submit));
        self
    }
    pub fn on_stop<R: HandlerResult>(mut self, stop: impl Fn() -> R + 'static) -> Self {
        self.on_stop = Some(callback(move |_| stop()));
        self
    }
    pub fn stop_shown(mut self, shown: impl IntoValue<bool>) -> Self {
        self.stop_shown = shown.into_value();
        self
    }
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn placeholder(mut self, placeholder: impl IntoText) -> Self {
        self.placeholder = placeholder.into_text_value();
        self
    }
    pub fn density(mut self, density: ChatDensity) -> Self {
        self.density = density;
        self
    }
    pub fn elevation(mut self, elevation: ComposerElevation) -> Self {
        self.elevation = elevation;
        self
    }
    pub fn status(mut self, status: impl IntoValue<Option<ComposerStatus>>) -> Self {
        self.status = status.into_value();
        self
    }
    /// Enable navigating submitted messages at the edges of the draft with Up and Down.
    pub fn history(mut self, enabled: bool) -> Self {
        self.history = enabled;
        self
    }

    pub fn status_position(mut self, position: ComposerStatusPosition) -> Self {
        self.status_position = position;
        self
    }
    pub fn drawer<V: IntoView>(mut self, drawer: impl FnOnce() -> V) -> Self {
        self.drawer = Some(drawer().into_view());
        self
    }
    pub fn header_actions<V: IntoView>(mut self, content: impl FnOnce() -> V) -> Self {
        self.header_actions = Some(content().into_view());
        self
    }
    pub fn header_context<V: IntoView>(mut self, content: impl FnOnce() -> V) -> Self {
        self.header_context = Some(content().into_view());
        self
    }
    pub fn footer_actions<V: IntoView>(mut self, content: impl FnOnce() -> V) -> Self {
        self.footer_actions = Some(content().into_view());
        self
    }
    pub fn send_actions<V: IntoView>(mut self, content: impl FnOnce() -> V) -> Self {
        self.send_actions = Some(content().into_view());
        self
    }
}

impl IntoView for ChatComposer {
    fn into_view(self) -> View {
        let Self {
            root,
            value,
            on_submit,
            on_stop,
            stop_shown,
            disabled,
            placeholder,
            density,
            elevation,
            drawer,
            header_actions,
            header_context,
            footer_actions,
            send_actions,
            status,
            status_position,
            history,
        } = self;
        let history = history.then(|| Rc::new(RefCell::new(ComposerHistory::default())));
        let sent_history = history.clone();
        let send = callback(move |()| {
            if disabled.get_untracked() || stop_shown.get_untracked() {
                return;
            }
            let draft = value.get_untracked();
            let message = draft.trim();
            if let Some(submit) = on_submit.filter(|_| !message.is_empty()) {
                let message = message.to_owned();
                if let Some(history) = &sent_history {
                    let mut history = history.borrow_mut();
                    history.submissions.push(message.clone());
                    history.recall = None;
                }
                value.set(String::new());
                submit.call(message);
            }
        });
        let input_ref = node_ref();
        let input =
            chat_composer_input(message("soft.chat_composer.input", "Message input"), value)
                .placeholder(placeholder)
                .disabled(disabled)
                .node_ref(input_ref)
                .forward_keys(vec![Key::Enter])
                .on_key(move |event: KeyEvent| {
                    if event.key == Key::Enter && event.modifiers == Modifiers::default() {
                        send.call(());
                        Handled::Yes
                    } else if event.modifiers == Modifiers::default()
                        && matches!(event.key, Key::ArrowUp | Key::ArrowDown)
                        && let Some(history) = &history
                        && let Some(editor) = input_ref.edit_state()
                        && let Some(mut recalled) =
                            history.borrow_mut().navigate(event.key, &editor)
                    {
                        recalled.move_to(0, false);
                        recalled.move_to(recalled.value().len(), true);
                        input_ref.set_edit_state(recalled);
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                });
        let send_button = chat_send_button()
            .stop_shown(stop_shown)
            .disabled(move || {
                if stop_shown.get() {
                    on_stop.is_none()
                } else {
                    disabled.get()
                        || on_submit.is_none()
                        || value.with(|value| value.trim().is_empty())
                }
            })
            .on_send(move || send.call(()))
            .on_stop(move || {
                if let Some(stop) = on_stop {
                    stop.call(());
                }
            })
            .size(if density == ChatDensity::Compact {
                ControlSize::Small
            } else {
                ControlSize::Medium
            });
        let header = row((header_context, header_actions))
            .style(style().justify_content(Justify::SpaceBetween));
        let footer = row((
            footer_actions,
            node().style(style().grow(1)),
            send_actions,
            send_button,
        ))
        .style(style().gap(8).align_items(Align::Center));
        let compact = density == ChatDensity::Compact;
        let surface = node()
            .style(move || {
                let p = tokens();
                let mut surface = style()
                    .direction(Direction::Column)
                    .min_width(0)
                    .padding(if compact { 8 } else { 12 })
                    .gap(4)
                    .radius(if compact { 16 } else { 28 })
                    .background(p.colors.bg_surface)
                    .border(1, p.colors.border);
                if elevation == ComposerElevation::Low {
                    for shadow in &p.shadows.low {
                        surface = surface.shadow(*shadow);
                    }
                }
                surface
            })
            .child((header, input, footer));
        let status = show(
            move || status.with(Option::is_some),
            move || {
                text(move || {
                    status.with(|status| match status {
                        Some(ComposerStatus::Error(message) | ComposerStatus::Warning(message)) => {
                            message.clone()
                        }
                        None => String::new(),
                    })
                })
                .role(Role::Status)
                .style(move || {
                    let p = tokens();
                    style().font_size(12).padding_x(8).color(
                        if status.with(|status| matches!(status, Some(ComposerStatus::Error(_)))) {
                            p.colors.text_error
                        } else {
                            p.colors.text_warning
                        },
                    )
                })
            },
        );
        let root = root.base_style(style().direction(Direction::Column).width(pct(100)).gap(4));
        match status_position {
            ComposerStatusPosition::Top => root.child((status, drawer, surface)),
            ComposerStatusPosition::Bottom => root.child((drawer, surface, status)),
        }
        .into_view()
    }
}

#[derive(Default)]
struct ComposerHistory {
    submissions: Vec<String>,
    recall: Option<(usize, String)>,
}

impl ComposerHistory {
    fn navigate(&mut self, key: Key, editor: &EditState) -> Option<EditState> {
        let text = editor.value();
        let selection = editor.selection();
        let at_start = selection.start == 0;
        let at_end = selection.end == text.len();
        let replacement = match key {
            Key::ArrowUp if at_start && (selection.is_empty() || at_end) => {
                let last = self.submissions.len().checked_sub(1)?;
                let (index, _) = self
                    .recall
                    .get_or_insert_with(|| (last + 1, text.to_owned()));
                *index = index.saturating_sub(1);
                &self.submissions[*index]
            }
            Key::ArrowDown if at_end && (selection.is_empty() || at_start) => {
                let (index, _) = self.recall.as_mut()?;
                *index += 1;
                match self.submissions.get(*index) {
                    Some(submission) => submission,
                    None => return self.recall.take().map(|(_, draft)| EditState::new(&draft)),
                }
            }
            _ => return None,
        };
        Some(EditState::new(replacement))
    }
}

pub struct ChatComposerDrawer {
    root: Node,
    content: View,
    count: Option<Value<usize>>,
    label: Value<String>,
    collapsed: Option<Binding<bool>>,
    summary: Option<View>,
}

pub fn chat_composer_drawer(content: impl IntoView) -> ChatComposerDrawer {
    ChatComposerDrawer {
        root: node(),
        content: content.into_view(),
        count: None,
        label: message("soft.chat_composer_drawer.items", "Items"),
        collapsed: None,
        summary: None,
    }
}

impl NodeBuilder for ChatComposerDrawer {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatComposerDrawer {
    pub fn count(mut self, count: impl IntoValue<usize>) -> Self {
        self.count = Some(count.into_value());
        self
    }
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = label.into_text_value();
        self
    }
    pub fn collapsed(mut self, collapsed: impl Into<Binding<bool>>) -> Self {
        self.collapsed = Some(collapsed.into());
        self
    }
    pub fn collapsed_summary<V: IntoView>(mut self, summary: impl FnOnce() -> V) -> Self {
        self.summary = Some(summary().into_view());
        self
    }
}

impl IntoView for ChatComposerDrawer {
    fn into_view(self) -> View {
        let Self {
            root,
            content,
            count,
            label,
            collapsed,
            summary,
        } = self;
        let root = root.base_style(move || {
            let p = tokens();
            style()
                .direction(Direction::Column)
                .padding_x(16)
                .padding_y(12)
                .gap(8)
                .radius_top_left(28)
                .radius_top_right(28)
                .background(p.colors.bg_muted)
                .color(p.colors.text_secondary)
        });
        let Some(count) = count else {
            return root.child(content).into_view();
        };
        let collapsed = collapsed.unwrap_or_else(|| signal(false).into());
        let controlled = node_ref();
        let button = node()
            .part(
                part(|state| state)
                    .role(Role::Button)
                    .focusable(true)
                    .controls(controlled)
                    .name(label)
                    .expanded(move || !collapsed.get())
                    .on_press(move || collapsed.set(!collapsed.get_untracked())),
            )
            .base_style(move || {
                let p = tokens();
                style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(8)
                    .padding_y(2)
                    .font_size(14)
                    .line_height(20.0 / 14.0)
                    .hover(style().color(p.colors.text_primary))
                    .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
            })
            .child((
                text(move || count.get().to_string()),
                text(label),
                text(move || if collapsed.get() { "›" } else { "⌄" }),
            ));
        let content = node().node_ref(controlled).hidden(collapsed).child(content);
        let summary = summary.map(|summary| node().hidden(move || !collapsed.get()).child(summary));
        root.child((button, summary, content)).into_view()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DictationState {
    pub supported: bool,
    pub listening: bool,
    pub volume: f32,
    pub bands: Vec<f32>,
}

pub struct ChatDictationButton {
    root: Node,
    state: Value<DictationState>,
    on_toggle: Option<Callback<()>>,
    size: ControlSize,
    hide_when_unsupported: bool,
    label: Value<String>,
}

pub fn chat_dictation_button(state: impl IntoValue<DictationState>) -> ChatDictationButton {
    ChatDictationButton {
        root: node(),
        state: state.into_value(),
        on_toggle: None,
        size: ControlSize::Medium,
        hide_when_unsupported: true,
        label: message("soft.chat_dictation_button.label", "Dictation"),
    }
}

impl NodeBuilder for ChatDictationButton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatDictationButton {
    pub fn on_toggle<R: HandlerResult>(mut self, toggle: impl Fn() -> R + 'static) -> Self {
        self.on_toggle = Some(callback(move |_| toggle()));
        self
    }
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    pub fn hide_when_unsupported(mut self, hide: bool) -> Self {
        self.hide_when_unsupported = hide;
        self
    }
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = label.into_text_value();
        self
    }
}

impl IntoView for ChatDictationButton {
    fn into_view(self) -> View {
        let Self {
            root,
            state,
            on_toggle,
            size,
            hide_when_unsupported,
            label,
        } = self;
        let side = control_side(size);
        let listening = move || state.with(|state| state.listening);
        let bars = (0..5)
            .map(|index| {
                node().style(move || {
                    let p = tokens();
                    let (band, volume) = state.with(|state| {
                        (state.bands.get(index).copied().unwrap_or(0.0), state.volume)
                    });
                    let level = (band.max(0.0) / 0.2).sqrt().min(1.0);
                    let height = if side <= 28 { 14.0 } else { 18.0 };
                    style()
                        .size(2.5, height * (0.08 + level * 0.92))
                        .radius(2)
                        .background(if volume >= 0.3 {
                            p.colors.error
                        } else {
                            p.colors.accent
                        })
                })
            })
            .collect::<Vec<_>>();
        let icon = text("🎙").hidden(listening);
        let waveform = row(bars)
            .hidden(move || !listening())
            .style(style().gap(2).align_items(Align::Center));
        root.part(
            part(|state| state)
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .checked(listening)
                .disabled(move || !state.with(|state| state.supported))
                .on_press(move || {
                    if let Some(toggle) = on_toggle {
                        toggle.call(());
                    }
                }),
        )
        .hidden(move || hide_when_unsupported && !state.with(|state| state.supported))
        .base_style(move || {
            let p = tokens();
            style()
                .direction(Direction::Row)
                .size(side, side)
                .radius(side as f32 / 2.0)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .color(p.colors.text_primary)
                .hover(style().background(p.colors.overlay_hover))
                .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
        })
        .child((icon, waveform))
        .into_view()
    }
}
