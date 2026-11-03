//! Role: Chat tools, token display and grapheme-safe streaming text.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::{SpinnerShade, SpinnerSize, spinner, tokens};
use kivact::control::{Node, NodeBuilder, Span, column, icon, node, rich_text, row, span, text};
use kivact::i18n::{message, message_with};
use kivact::interaction::use_disclosure;
use kivact::primitive::{EditState, NodeExt};
use kivact::style::{Align, Direction, style};
use kivact::task::{Task, sleep, spawn};
use kivact::view::{
    Handled, IntoText, IntoValue, IntoView, Key, KeyEvent, Role, View, for_each, node_ref, show,
    show_some,
};
use kivact::{Binding, Value, binding, effect, memo, signal};

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
        .radius(p.radius.element)
        .color(p.colors.text_secondary)
        .hover(style().background(p.colors.overlay_hover))
        .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
}

fn tool_summary(call: Value<ToolCall>, visible: Value<bool>) -> (Node, Value<String>) {
    let pending = message("soft.chat.tool_calls.pending", "Pending");
    let running = message("soft.chat.tool_calls.running", "Running");
    let complete = message("soft.chat.tool_calls.complete", "Complete");
    let failed = message("soft.chat.tool_calls.failed", "Failed");
    let status = memo(move || call.with(|call| call.status));
    let status_label = memo(move || match status.get() {
        ToolStatus::Pending => pending.get(),
        ToolStatus::Running => running.get(),
        ToolStatus::Complete => complete.get(),
        ToolStatus::Error => failed.get(),
    });
    let pending = show(
        move || visible.get() && status.get() == ToolStatus::Pending,
        move || {
            spinner()
                .size(SpinnerSize::Sm)
                .shade(SpinnerShade::Subtle)
                .name(status_label)
        },
    );
    let running = show(
        move || visible.get() && status.get() == ToolStatus::Running,
        move || spinner().size(SpinnerSize::Sm).name(status_label),
    );
    let result = text(move || {
        let glyph = if status.get() == ToolStatus::Error {
            "!"
        } else {
            "✓"
        };
        format!("{glyph} {}", status_label.get())
    })
    .name(status_label)
    .hidden(move || matches!(status.get(), ToolStatus::Pending | ToolStatus::Running))
    .style(move || {
        let p = tokens();
        style()
            .shrink(0)
            .single_line()
            .color(if status.get() == ToolStatus::Error {
                p.colors.text_error
            } else {
                p.colors.text_success
            })
    });
    let label = text(move || call.with(|call| call.name.clone())).style(move || {
        style()
            .font_family(tokens().typography.family_code)
            .weight(500)
            .min_width(24)
            .single_line()
    });
    let target = text(move || call.with(|call| call.target.clone().unwrap_or_default()))
        .hidden(move || call.with(|call| call.target.is_none()))
        .style(style().shrink(10).min_width(0).single_line());
    let duration = text(move || {
        call.with(|call| {
            call.duration
                .map(|duration| format!("{:.2}s", duration.as_secs_f64()))
                .unwrap_or_default()
        })
    })
    .hidden(move || {
        call.with(|call| call.status != ToolStatus::Complete || call.duration.is_none())
    })
    .style(style().single_line().shrink(0));
    let additions = text(move || call.with(|call| format!("+{}", call.additions)))
        .hidden(move || call.with(|call| call.additions == 0))
        .style(move || {
            style()
                .color(tokens().colors.text_success)
                .single_line()
                .shrink(0)
        });
    let deletions = text(move || call.with(|call| format!("−{}", call.deletions)))
        .hidden(move || call.with(|call| call.deletions == 0))
        .style(move || {
            style()
                .color(tokens().colors.text_error)
                .single_line()
                .shrink(0)
        });
    (
        row((
            pending, running, result, label, target, additions, deletions, duration,
        ))
        .style(move || {
            style()
                .gap(6)
                .min_height(24)
                .min_width(0)
                .align_items(Align::Center)
                .color(tokens().colors.text_secondary)
        }),
        status_label,
    )
}

fn tool_chevron(open: impl IntoValue<bool>) -> Node {
    let open = open.into_value();
    icon("chevron-down").style(move || {
        let p = tokens();
        style()
            .size(14, 14)
            .shrink(0)
            .color(p.colors.icon_secondary)
            .rotate(if open.get() { 180 } else { 0 })
            .transition(
                kivact::style::TransitionProp::Paint,
                kivact::motion::Transition::tween(125.0, p.motion.ease_standard),
            )
    })
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
        let multiple = memo(move || calls.with(|calls| calls.len() > 1));
        let rows_open = binding(
            move || !multiple.get() || open.get(),
            move |shown| open.set(shown),
        );
        let disclosure = use_disclosure(rows_open);
        let title = label.unwrap_or_else(|| {
            message_with(
                "soft.chat.tool_calls.group_label",
                "{count} tool calls",
                move || [("count", calls.with(Vec::len).to_string())],
            )
        });
        let latest = memo(move || {
            if multiple.get() && !open.get() {
                calls.with(|calls| calls.last().cloned())
            } else {
                None
            }
        });
        let preview = show_some(latest, move |call| {
            let (summary, _) = tool_summary(call, true.into_value());
            row((
                summary.style(style().grow(1).min_width(0)),
                icon("wrench").style(style().size(12, 12).shrink(0)),
                text(move || calls.with(|calls| calls.len().to_string())).style(style().shrink(0)),
            ))
            .style(
                style()
                    .gap(4)
                    .align_items(Align::Center)
                    .grow(1)
                    .min_width(0),
            )
        });
        let heading = show(
            move || open.get() && multiple.get(),
            move || {
                row((icon("wrench").style(style().size(16, 16)), text(title))).style(
                    style()
                        .gap(6)
                        .align_items(Align::Center)
                        .grow(1)
                        .min_width(0),
                )
            },
        );
        let trigger = node()
            .part(disclosure.trigger().name(title))
            .hidden(move || !multiple.get())
            .base_style(disclosure_style)
            .child((preview, heading, tool_chevron(open)));
        let rows_visible = rows_open.into_value();
        let items = for_each(
            calls,
            |call| call.key.clone(),
            move |_, call| {
                let (summary, status) = tool_summary(call, rows_visible);
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
                .style(move || style().padding_left(22).color(tokens().colors.text_error));
                let summary = if let Some(detail) = &detail {
                    let expanded = signal(false);
                    let disclosure = use_disclosure(expanded);
                    let detail_label = message_with(
                        "soft.chat.tool_calls.detail",
                        "Details for {name}",
                        move || [("name", call.with(|call| call.name.clone()))],
                    );
                    let trigger_ref = node_ref();
                    let trigger = node()
                        .part(disclosure.trigger().name(detail_label).description(status))
                        .node_ref(trigger_ref)
                        .base_style(disclosure_style)
                        .child((summary.style(style().grow(1)), tool_chevron(expanded)));
                    let content = node()
                        .role(Role::Group)
                        .name(detail_label)
                        .part(disclosure.content())
                        .on_key(move |event: KeyEvent| {
                            if event.key == Key::Escape {
                                expanded.set(false);
                                trigger_ref.restore_focus();
                                Handled::Yes
                            } else {
                                Handled::No
                            }
                        })
                        .style(style().padding_left(22).padding_bottom(8))
                        .child(detail(call));
                    column((trigger, error, content))
                } else {
                    column((summary, error)).style(style().padding_x(6))
                };
                summary.role(Role::ListItem)
            },
        );
        let items = column(items)
            .role(Role::List)
            .name(title)
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
