//! Role: Chat tools, token display and grapheme-safe streaming text.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::tokens;
use kivact::control::{Node, NodeBuilder, Span, column, node, rich_text, row, span, text};
use kivact::i18n::{message, message_with};
use kivact::interaction::use_disclosure;
use kivact::primitive::{EditState, NodeExt};
use kivact::style::{Align, Direction, style};
use kivact::task::{Task, sleep, spawn};
use kivact::view::{IntoText, IntoValue, IntoView, Role, View, for_each};
use kivact::{Binding, Value, effect, signal};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolStatus {
    Pending,
    Running,
    #[default]
    Complete,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCall {
    pub key: String,
    pub name: String,
    pub status: ToolStatus,
    pub target: Option<String>,
    pub duration: Option<Duration>,
    pub additions: u32,
    pub deletions: u32,
    pub error: Option<String>,
}

impl ToolCall {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            status: ToolStatus::Complete,
            target: None,
            duration: None,
            additions: 0,
            deletions: 0,
            error: None,
        }
    }
    pub fn status(mut self, status: ToolStatus) -> Self {
        self.status = status;
        self
    }
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
    pub fn diff(mut self, additions: u32, deletions: u32) -> Self {
        self.additions = additions;
        self.deletions = deletions;
        self
    }
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.status = ToolStatus::Error;
        self.error = Some(error.into());
        self
    }
}

type ToolDetails = Box<dyn Fn(Value<ToolCall>) -> View>;

pub struct ChatToolCalls {
    root: Node,
    calls: Value<Vec<ToolCall>>,
    label: Option<Value<String>>,
    expanded: Option<Binding<bool>>,
    detail: Option<ToolDetails>,
}

pub fn chat_tool_calls(calls: impl IntoValue<Vec<ToolCall>>) -> ChatToolCalls {
    ChatToolCalls {
        root: node(),
        calls: calls.into_value(),
        label: None,
        expanded: None,
        detail: None,
    }
}

impl NodeBuilder for ChatToolCalls {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatToolCalls {
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }
    pub fn expanded(mut self, expanded: impl Into<Binding<bool>>) -> Self {
        self.expanded = Some(expanded.into());
        self
    }
    pub fn detail<V: IntoView>(mut self, detail: impl Fn(Value<ToolCall>) -> V + 'static) -> Self {
        self.detail = Some(Box::new(move |call| detail(call).into_view()));
        self
    }
}

fn disclosure_style() -> kivact::style::Style {
    let p = tokens();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(6)
        .padding_x(6)
        .min_height(24)
        .radius(p.radius.inner)
        .color(p.colors.text_secondary)
        .hover(style().background(p.colors.overlay_hover))
        .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
}

impl IntoView for ChatToolCalls {
    fn into_view(self) -> View {
        let Self {
            root,
            calls,
            label,
            expanded,
            detail,
        } = self;
        let open = expanded.unwrap_or_else(|| signal(false).into());
        let disclosure = use_disclosure(open);
        let count = label.map(|_| text(move || format!("({})", calls.with(Vec::len))));
        let title = label.unwrap_or_else(|| {
            message_with(
                "soft.chat.tool_calls.group_label",
                "{count} tool calls",
                move || [("count", calls.with(Vec::len).to_string())],
            )
        });
        let trigger = node()
            .part(disclosure.trigger().name(title))
            .base_style(disclosure_style)
            .child((
                text(move || if open.get() { "⌄" } else { "›" }),
                text(title),
                count,
            ));
        let items = for_each(
            calls,
            |call| call.key.clone(),
            move |_, call| {
                let pending = message("soft.chat.tool_calls.pending", "Pending");
                let running = message("soft.chat.tool_calls.running", "Running");
                let complete = message("soft.chat.tool_calls.complete", "Complete");
                let failed = message("soft.chat.tool_calls.failed", "Failed");
                let status_label = move || {
                    call.with(|call| match call.status {
                        ToolStatus::Pending => pending.get(),
                        ToolStatus::Running => running.get(),
                        ToolStatus::Complete => complete.get(),
                        ToolStatus::Error => failed.get(),
                    })
                };
                let status = text(move || {
                    let glyph = call.with(|call| match call.status {
                        ToolStatus::Pending | ToolStatus::Running => "◷",
                        ToolStatus::Complete => "✓",
                        ToolStatus::Error => "!",
                    });
                    format!("{glyph} {}", status_label())
                })
                .name(status_label)
                .style(move || {
                    let p = tokens();
                    let ink = call.with(|call| match call.status {
                        ToolStatus::Pending => p.colors.text_disabled,
                        ToolStatus::Running => p.colors.text_accent,
                        ToolStatus::Complete => p.colors.text_success,
                        ToolStatus::Error => p.colors.text_error,
                    });
                    style().color(ink)
                });
                let label =
                    text(move || call.with(|call| call.name.clone())).style(style().weight(600));
                let target =
                    text(move || call.with(|call| call.target.clone().unwrap_or_default()))
                        .hidden(move || call.with(|call| call.target.is_none()));
                let duration = text(move || {
                    call.with(|call| {
                        call.duration
                            .map(|d| format!("{:.2}s", d.as_secs_f64()))
                            .unwrap_or_default()
                    })
                })
                .hidden(move || call.with(|call| call.duration.is_none()));
                let diff = text(move || {
                    call.with(|call| format!("+{} −{}", call.additions, call.deletions))
                })
                .hidden(move || call.with(|call| call.additions == 0 && call.deletions == 0));
                let summary = row((status, label, target, duration, diff)).style(
                    style()
                        .min_height(24)
                        .gap(6)
                        .align_items(Align::Center)
                        .wrap(true),
                );
                let error = text(message_with(
                    "soft.chat.tool_calls.error",
                    "Error: {message}",
                    move || {
                        [(
                            "message",
                            call.with(|call| call.error.clone().unwrap_or_default()),
                        )]
                    },
                ))
                .hidden(move || call.with(|call| call.error.is_none()))
                .style(move || style().color(tokens().colors.text_error));
                let detail = detail.as_ref().map(|detail| {
                    let disclosure = use_disclosure(signal(false));
                    let detail_label = message_with(
                        "soft.chat.tool_calls.detail",
                        "Details for {name}",
                        move || [("name", call.with(|call| call.name.clone()))],
                    );
                    let trigger = node()
                        .part(disclosure.trigger().name(detail_label))
                        .base_style(disclosure_style)
                        .child(text(message("soft.chat.tool_calls.details", "Details")));
                    column((
                        trigger,
                        node()
                            .role(Role::Group)
                            .name(detail_label)
                            .part(disclosure.content())
                            .child(detail(call)),
                    ))
                    .style(style().gap(2))
                });
                column((summary, error, detail))
                    .role(Role::ListItem)
                    .style(style().gap(2).padding_x(6))
            },
        );
        let items = column(items)
            .role(Role::List)
            .part(disclosure.content())
            .style(style().gap(2));
        root.hidden(move || calls.with(Vec::is_empty))
            .base_style(
                style()
                    .direction(Direction::Column)
                    .gap(2)
                    .margin_top(8)
                    .font_size(12),
            )
            .child((trigger, items))
            .into_view()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatTokenTone {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatToken {
    pub value: String,
    pub label: String,
    pub tone: ChatTokenTone,
}

impl ChatToken {
    pub fn badge(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            tone: ChatTokenTone::Neutral,
        }
    }
    pub fn tone(mut self, tone: ChatTokenTone) -> Self {
        self.tone = tone;
        self
    }
}

fn token_colors(tone: ChatTokenTone) -> (kivact::style::Color, kivact::style::Color) {
    let p = tokens();
    match tone {
        ChatTokenTone::Neutral => (p.colors.neutral, p.colors.text_primary),
        ChatTokenTone::Info => (p.colors.info_muted, p.colors.on_info_muted),
        ChatTokenTone::Success => (p.colors.success_muted, p.colors.on_success_muted),
        ChatTokenTone::Warning => (p.colors.warning_muted, p.colors.on_warning_muted),
        ChatTokenTone::Error => (p.colors.error_muted, p.colors.on_error_muted),
    }
}

pub fn chat_token(token: ChatToken) -> Node {
    let tone = token.tone;
    text(token.label).base_style(move || {
        let (fill, ink) = token_colors(tone);
        style()
            .height(20)
            .padding_x(8)
            .radius(10)
            .font_size(12)
            .line_height(20.0 / 12.0)
            .weight(500)
            .single_line()
            .background(fill)
            .color(ink)
    })
}

fn tokenize(source: &str, tokens: &[ChatToken]) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut rest = source;
    while !rest.is_empty() {
        let next = tokens
            .iter()
            .filter(|token| !token.value.is_empty())
            .filter_map(|token| rest.find(&token.value).map(|index| (index, token)))
            .min_by_key(|(index, _)| *index);
        let Some((index, token)) = next else {
            spans.push(span(rest));
            break;
        };
        if index != 0 {
            spans.push(span(&rest[..index]));
        }
        let (fill, ink) = token_colors(token.tone);
        spans.push(
            span(&token.label)
                .font_size(12)
                .weight(500)
                .background(fill)
                .color(ink),
        );
        rest = &rest[index + token.value.len()..];
    }
    spans
}

pub fn chat_tokenized_text(source: impl IntoText, tokens: impl IntoValue<Vec<ChatToken>>) -> Node {
    let source = source.into_text_value();
    let tokens = tokens.into_value();
    rich_text(move || source.with(|source| tokens.with(|tokens| tokenize(source, tokens))))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StreamSpeed {
    #[default]
    Natural,
    Fast,
    Instant,
}

pub fn use_streaming_text(target: impl IntoText, streaming: impl IntoValue<bool>) -> Value<String> {
    streaming_text(target, streaming, StreamSpeed::Natural)
}

pub fn streaming_text(
    target: impl IntoText,
    streaming: impl IntoValue<bool>,
    speed: StreamSpeed,
) -> Value<String> {
    let target = target.into_text_value();
    let streaming = streaming.into_value();
    let visible = signal(String::new());
    let task = Rc::new(Cell::<Option<Task>>::new(None));
    effect(move || {
        let received = target.get();
        let animated =
            streaming.get() && speed != StreamSpeed::Instant && !kivact::platform::reduced_motion();
        if !animated || received.is_empty() {
            if let Some(task) = task.take() {
                task.cancel();
            }
            visible.set(received);
            return;
        }
        if !visible.with_untracked(|shown| received.starts_with(shown)) {
            visible.set(String::new());
        }
        if task.get().is_some() || visible.with_untracked(|shown| *shown == received) {
            return;
        }
        let task_slot = task.clone();
        let (delay_ms, budget) = match speed {
            StreamSpeed::Natural => (10, 10),
            StreamSpeed::Fast => (5, 4),
            StreamSpeed::Instant => unreachable!(),
        };
        task.set(Some(spawn(async move {
            let mut editor = EditState::default();
            let mut end = 0;
            loop {
                sleep(Duration::from_millis(delay_ms)).await;
                let received = target.get_untracked();
                if !streaming.get_untracked() || kivact::platform::reduced_motion() {
                    visible.set(received);
                    break;
                }
                if editor.value() != received {
                    let shown = visible.get_untracked();
                    end = if received.starts_with(&shown) {
                        shown.len()
                    } else {
                        0
                    };
                    editor = EditState::new(&received);
                    // A new chunk can extend the final grapheme of the previous chunk.
                    if end > 0 && end < received.len() {
                        end = editor.next_grapheme(editor.prev_grapheme(end));
                    }
                }
                let start = end;
                while end < received.len()
                    && (end == start || received[start..end].encode_utf16().count() < budget)
                {
                    end = editor.next_grapheme(end);
                }
                visible.set(received[..end].to_owned());
                if end == received.len() {
                    break;
                }
            }
            task_slot.set(None);
        })));
    });
    visible.into_value()
}
