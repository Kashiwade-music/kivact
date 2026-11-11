//! Role: Chat tools, token display and grapheme-safe streaming text.

use kivact::state::IntoBinding;
use std::time::Duration;

use crate::{SpinnerShade, SpinnerVariant};
use kivact::custom::EditState;
use kivact::i18n::{message, message_with};
use kivact::interaction::use_disclosure;
use kivact::motion::Transition;
use kivact::state::Binding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Machine;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::machine;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::TransitionProp;
use kivact::style::style;
use kivact::ui::{ControlSize, spinner};
use kivact::view::Element;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::Node;
use kivact::view::Role;
use kivact::view::Span;
use kivact::view::View;
use kivact::view::column;
use kivact::view::for_each;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::rich_text;
use kivact::view::row;
use kivact::view::show;
use kivact::view::show_some;
use kivact::view::span;
use kivact::view::text;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Execution status supplied by the application for a tool call.
pub enum ToolStatus {
    /// The tool has not started executing.
    Pending,
    /// The tool is executing.
    Running,
    /// The tool completed successfully, with its duration when reported.
    Complete {
        /// The reported execution duration.
        duration: Option<Duration>,
    },
    /// The tool failed, with the failure details when reported.
    Error {
        /// The failure details supplied by the application.
        message: Option<String>,
    },
}

impl Default for ToolStatus {
    fn default() -> Self {
        Self::Complete { duration: None }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Application-owned identity, execution result and optional diagnostics for one tool call.
pub struct ToolCall {
    /// Stable identity used when reconciling tool-call rows.
    pub key: String,
    /// Tool name shown in the row.
    pub name: String,
    /// Current execution status and its reported details.
    pub status: ToolStatus,
    // xtask-allow(state-fields): option: application-supplied target text shown beside the tool name in every status; absent hides the target label.
    /// Optional target description shown beside the tool name.
    pub target: Option<String>,
    /// Number of additions shown in the change summary.
    pub additions: u32,
    /// Number of deletions shown in the change summary.
    pub deletions: u32,
}

impl ToolCall {
    /// Create a completed tool-call record with no target, duration or changes.
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            status: ToolStatus::Complete { duration: None },
            target: None,
            additions: 0,
            deletions: 0,
        }
    }
    /// Set the execution status and its reported details.
    pub fn status(mut self, status: ToolStatus) -> Self {
        self.status = status;
        self
    }
    /// Set the target description displayed for this record.
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
    /// Set the addition and deletion counts in the change summary.
    pub fn diff(mut self, additions: u32, deletions: u32) -> Self {
        self.additions = additions;
        self.deletions = deletions;
        self
    }
}

type ToolDetails = kivact::view::ViewFn<Value<ToolCall>>;

/// A labelled collection of tool calls with keyboard-operable group and detail disclosures.
pub struct ChatToolCalls {
    root: Node,
    calls: Value<Vec<ToolCall>>,
    label: Option<Value<String>>,
    expanded: Option<Binding<bool>>,
    detail: Option<ToolDetails>,
}

/// Render reactive tool-call records with stable row identities.
pub fn chat_tool_calls(items: impl IntoValue<Vec<ToolCall>>) -> ChatToolCalls {
    ChatToolCalls {
        root: node(),
        calls: items.into_value(),
        label: None,
        expanded: None,
        detail: None,
    }
}

impl Element for ChatToolCalls {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl ChatToolCalls {
    /// Set the accessible label of the tool-call group.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }
    /// Bind the group disclosure to application-owned expansion state.
    pub fn expanded(mut self, expanded: impl IntoBinding<bool>) -> Self {
        self.expanded = Some(expanded.into_binding());
        self
    }
    /// Render additional content for an expanded tool-call row.
    pub fn detail<V: IntoView>(mut self, detail: impl Fn(Value<ToolCall>) -> V + 'static) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

fn disclosure_style() -> kivact::style::Style {
    let p = kivact::style::use_tokens::<crate::Tokens>();
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
    let (pending, running, result, status_label) = tool_status(call, visible);
    let (label, target, duration) = tool_labels(call);
    let (additions, deletions) = tool_diff(call);
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
                .color(
                    kivact::style::use_tokens::<crate::Tokens>()
                        .colors
                        .text_secondary,
                )
        }),
        status_label,
    )
}

fn tool_status(
    call: Value<ToolCall>,
    visible: Value<bool>,
) -> (impl IntoView, impl IntoView, Node, Value<String>) {
    let pending = message("soft.chat.tool_calls.pending", "Pending");
    let running = message("soft.chat.tool_calls.running", "Running");
    let complete = message("soft.chat.tool_calls.complete", "Complete");
    let failed = message("soft.chat.tool_calls.failed", "Failed");
    let status = memo(move || call.with(|call| call.status.clone()));
    let status_label = memo(move || match status.get() {
        ToolStatus::Pending => pending.get(),
        ToolStatus::Running => running.get(),
        ToolStatus::Complete { .. } => complete.get(),
        ToolStatus::Error { .. } => failed.get(),
    });
    let pending = show(
        move || visible.get() && matches!(status.get(), ToolStatus::Pending),
        move || {
            spinner()
                .size(ControlSize::ExtraSmall)
                .variant(SpinnerVariant {
                    shade: SpinnerShade::Subtle,
                    ..Default::default()
                })
                .name(status_label)
        },
    );
    let running = show(
        move || visible.get() && matches!(status.get(), ToolStatus::Running),
        move || spinner().size(ControlSize::ExtraSmall).name(status_label),
    );
    let result = text(move || {
        let glyph = if matches!(status.get(), ToolStatus::Error { .. }) {
            "!"
        } else {
            "✓"
        };
        format!("{glyph} {}", status_label.get())
    })
    .name(status_label)
    .hidden(move || matches!(status.get(), ToolStatus::Pending | ToolStatus::Running))
    .style(move || {
        let p = kivact::style::use_tokens::<crate::Tokens>();
        style()
            .shrink(0)
            .single_line()
            .color(if matches!(status.get(), ToolStatus::Error { .. }) {
                p.colors.text_error
            } else {
                p.colors.text_success
            })
    });
    (pending, running, result, status_label)
}

fn tool_labels(call: Value<ToolCall>) -> (Node, Node, Node) {
    let label = text(move || call.with(|call| call.name.clone())).style(move || {
        style()
            .font_family(
                kivact::style::use_tokens::<crate::Tokens>()
                    .typography
                    .family_code,
            )
            .weight(500)
            .min_width(24)
            .single_line()
    });
    let target = text(move || call.with(|call| call.target.clone().unwrap_or_default()))
        .hidden(move || call.with(|call| call.target.is_none()))
        .style(style().shrink(10).min_width(0).single_line());
    let duration = text(move || {
        call.with(|call| match &call.status {
            ToolStatus::Complete {
                duration: Some(duration),
            } => format!("{:.2}s", duration.as_secs_f64()),
            _ => String::new(),
        })
    })
    .hidden(move || {
        call.with(|call| !matches!(&call.status, ToolStatus::Complete { duration: Some(_) }))
    })
    .style(style().single_line().shrink(0));
    (label, target, duration)
}

fn tool_diff(call: Value<ToolCall>) -> (Node, Node) {
    let additions = text(move || call.with(|call| format!("+{}", call.additions)))
        .hidden(move || call.with(|call| call.additions == 0))
        .style(move || {
            style()
                .color(
                    kivact::style::use_tokens::<crate::Tokens>()
                        .colors
                        .text_success,
                )
                .single_line()
                .shrink(0)
        });
    let deletions = text(move || call.with(|call| format!("−{}", call.deletions)))
        .hidden(move || call.with(|call| call.deletions == 0))
        .style(move || {
            style()
                .color(
                    kivact::style::use_tokens::<crate::Tokens>()
                        .colors
                        .text_error,
                )
                .single_line()
                .shrink(0)
        });
    (additions, deletions)
}

fn tool_chevron(open: impl IntoValue<bool>) -> Node {
    let open = open.into_value();
    icon("chevron-down").style(move || {
        let p = kivact::style::use_tokens::<crate::Tokens>();
        style()
            .size(14, 14)
            .shrink(0)
            .color(p.colors.icon_secondary)
            .rotate(if open.get() { 180 } else { 0 })
            .transition(
                kivact::style::TransitionProp::All,
                kivact::motion::Transition::tween(125.0, p.motion.ease_standard.clone()),
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
        let open = expanded.unwrap_or_else(|| signal(false).into_binding());
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
        let trigger = tool_group_trigger(calls, open, multiple, disclosure, title);
        let items = tool_group_items(calls, disclosure, title, detail);
        root.hidden(move || calls.with(Vec::is_empty))
            .with_node(move |node| {
                node.base_style(
                    style()
                        .direction(Direction::Column)
                        .gap(2)
                        .margin_top(8)
                        .font_size(12),
                )
            })
            .child((trigger, items))
            .into_view()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Semantic color treatment of a substituted chat token.
pub enum ChatTokenTone {
    #[default]
    /// Use neutral token colors.
    Neutral,
    /// Use informational token colors.
    Info,
    /// Use successful token colors.
    Success,
    /// Use warning token colors.
    Warning,
    /// Use error token colors.
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A source substring and its styled replacement label.
pub struct ChatToken {
    /// Nonempty source substring to replace when tokenizing text.
    pub value: String,
    /// Visible replacement label.
    pub label: String,
    /// Semantic color treatment of the replacement.
    pub tone: ChatTokenTone,
}

impl ChatToken {
    /// Create a neutral token replacing the supplied source substring with a label.
    pub fn badge(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            tone: ChatTokenTone::Neutral,
        }
    }
    /// Set the semantic color treatment of this token.
    pub fn tone(mut self, tone: ChatTokenTone) -> Self {
        self.tone = tone;
        self
    }
}

fn token_colors(tone: ChatTokenTone) -> (kivact::style::Color, kivact::style::Color) {
    let p = kivact::style::use_tokens::<crate::Tokens>();
    match tone {
        ChatTokenTone::Neutral => (p.colors.neutral, p.colors.text_primary),
        ChatTokenTone::Info => (p.colors.info_muted, p.colors.on_info_muted),
        ChatTokenTone::Success => (p.colors.success_muted, p.colors.on_success_muted),
        ChatTokenTone::Warning => (p.colors.warning_muted, p.colors.on_warning_muted),
        ChatTokenTone::Error => (p.colors.error_muted, p.colors.on_error_muted),
    }
}

/// Render a standalone styled chat token label.
pub fn chat_token(token: ChatToken) -> impl kivact::view::Element {
    let tone = token.tone;
    text(token.label).with_node(move |node| {
        node.base_style(move || {
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
            span(token.label.clone())
                .font_size(12)
                .weight(500)
                .background(fill)
                .color(ink),
        );
        rest = &rest[index + token.value.len()..];
    }
    spans
}

/// Render text with matching source substrings replaced by styled token labels.
pub fn chat_tokenized_text(
    source: impl IntoText,
    tokens: impl IntoValue<Vec<ChatToken>>,
) -> impl kivact::view::Element {
    let source = source.into_text_value();
    let tokens = tokens.into_value();
    rich_text(move || source.with(|source| tokens.with(|tokens| tokenize(source, tokens))))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Reveal pacing used while streamed text is arriving.
pub enum StreamSpeed {
    #[default]
    /// Reveal text in the standard paced batches.
    Natural,
    /// Use shorter ticks and smaller reveal batches.
    Fast,
    /// Show the complete target immediately.
    Instant,
}

/// Reveal a reactive target at the natural pace while streaming is active.
pub fn use_streaming_text(target: impl IntoText, streaming: impl IntoValue<bool>) -> Value<String> {
    streaming_text(target, streaming, StreamSpeed::Natural)
}

/// Reveal a reactive target at the selected pace, respecting reduced motion.
pub fn streaming_text(
    target: impl IntoText,
    streaming: impl IntoValue<bool>,
    speed: StreamSpeed,
) -> Value<String> {
    let target = target.into_text_value();
    let streaming = streaming.into_value();
    let stream = machine(
        StreamSnapshot {
            source: String::new(),
            phase: Stream::Idle,
        },
        move || StreamInput {
            target: target.get(),
            animated: streaming.get()
                && speed != StreamSpeed::Instant
                && !kivact::platform::is_motion_reduced().get(),
            speed,
        },
        |_, ()| {},
    );
    stream.feed(
        move || {
            (
                target.get(),
                streaming.get(),
                kivact::platform::is_motion_reduced().get(),
            )
        },
        |_| StreamEvent::Changed,
    );
    stream.send(StreamEvent::Changed);
    memo(move || {
        let state = stream.get();
        match state.phase {
            Stream::Idle => String::new(),
            Stream::Revealing { shown } => state.source[..shown].to_owned(),
            Stream::Done => state.source,
        }
    })
    .into_value()
}

#[derive(Clone, PartialEq)]
enum Stream {
    Idle,
    Revealing { shown: usize },
    Done,
}
#[derive(Clone, PartialEq)]
struct StreamSnapshot {
    source: String,
    phase: Stream,
}
enum StreamEvent {
    Changed,
    Tick,
}
struct StreamInput {
    target: String,
    animated: bool,
    speed: StreamSpeed,
}
impl Machine for StreamInput {
    type State = StreamSnapshot;
    type Event = StreamEvent;
    type Effect = ();
    fn transition(
        &self,
        state: &StreamSnapshot,
        event: StreamEvent,
    ) -> Step<StreamSnapshot, StreamEvent, ()> {
        let next = |phase| StreamSnapshot {
            source: self.target.clone(),
            phase,
        };
        if !self.animated || self.target.is_empty() {
            return Step::to(next(Stream::Done)).cancel(StreamEvent::Tick);
        }
        let previous = match state.phase {
            Stream::Revealing { shown } => &state.source[..shown],
            Stream::Idle => "",
            Stream::Done => &state.source,
        };
        let mut shown = if self.target.starts_with(previous) {
            previous.len()
        } else {
            0
        };
        let editor = EditState::new(&self.target);
        if matches!(event, StreamEvent::Tick) {
            if shown > 0 && shown < self.target.len() {
                shown = editor.next_grapheme(editor.prev_grapheme(shown));
            }
            let start = shown;
            let budget = if self.speed == StreamSpeed::Fast {
                4
            } else {
                10
            };
            while shown < self.target.len()
                && (shown == start || self.target[start..shown].encode_utf16().count() < budget)
            {
                shown = editor.next_grapheme(shown);
            }
        }
        if shown == self.target.len() {
            return Step::to(next(Stream::Done)).cancel(StreamEvent::Tick);
        }
        if matches!(event, StreamEvent::Changed) && matches!(state.phase, Stream::Revealing { .. })
        {
            return Step::to(next(Stream::Revealing { shown }));
        }
        Step::to(next(Stream::Revealing { shown })).after(
            Duration::from_millis(if self.speed == StreamSpeed::Fast {
                5
            } else {
                10
            }),
            StreamEvent::Tick,
        )
    }
}

fn tool_group_trigger(
    calls: Value<Vec<ToolCall>>,
    open: Binding<bool>,
    multiple: Value<bool>,
    disclosure: kivact::interaction::DisclosureApi,
    title: Value<String>,
) -> Node {
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
    node()
        .with_node(move |node| node.part(disclosure.trigger()))
        .name(title)
        .hidden(move || !multiple.get())
        .with_node(move |node| node.base_style(disclosure_style))
        .child((preview, heading, tool_chevron(open)))
}

fn tool_group_items(
    calls: Value<Vec<ToolCall>>,
    disclosure: kivact::interaction::DisclosureApi,
    title: Value<String>,
    detail: Option<ToolDetails>,
) -> impl IntoView {
    let motion = kivact::style::use_tokens::<crate::Tokens>().motion.clone();
    show(disclosure.is_open(), move || {
        let detail = detail.clone();
        let items = for_each(
            calls,
            |call| call.key.clone(),
            move |call| tool_row(call, disclosure.is_open(), detail.clone()),
        );
        column(
            column(items)
                .with_node(move |node| node.role(Role::List))
                .name(title)
                .style(style().gap(2).shrink(0)),
        )
        .with_node(move |node| node.role(Role::Group))
        .with_node(move |node| node.part(disclosure.content()))
        .style(
            style()
                .enter_size(kivact::view::Orientation::Vertical)
                .exit_size(kivact::view::Orientation::Vertical)
                .transition(
                    TransitionProp::Size,
                    Transition::tween(motion.medium as f64, motion.ease_standard.clone()),
                ),
        )
    })
}

fn tool_row(call: Value<ToolCall>, rows_visible: Value<bool>, detail: Option<ToolDetails>) -> Node {
    let (summary, status) = tool_summary(call, rows_visible);
    let error = text(message_with(
        "soft.chat.tool_calls.error",
        "Error: {message}",
        move || {
            [(
                "message",
                call.with(|call| match &call.status {
                    ToolStatus::Error { message } => message.clone().unwrap_or_default(),
                    _ => String::new(),
                }),
            )]
        },
    ))
    .hidden(move || {
        call.with(|call| !matches!(&call.status, ToolStatus::Error { message: Some(_) }))
    })
    .style(move || {
        style().padding_left(22).color(
            kivact::style::use_tokens::<crate::Tokens>()
                .colors
                .text_error,
        )
    });
    let summary = if let Some(detail) = &detail {
        let expanded = signal(false);
        // The detail content is visible only while the group shows its rows.
        let content_visible = memo(move || expanded.get() && rows_visible.get());
        let disclosure = use_disclosure(expanded).disabled(move || !rows_visible.get());
        let detail_label = message_with(
            "soft.chat.tool_calls.detail",
            "Details for {name}",
            move || [("name", call.with(|call| call.name.clone()))],
        );
        let trigger_ref = node_ref();
        let trigger = node()
            .with_node(move |node| node.part(disclosure.trigger()))
            .name(detail_label)
            .description(status)
            .node_ref(trigger_ref)
            .with_node(move |node| node.base_style(disclosure_style))
            .child((summary.style(style().grow(1)), tool_chevron(expanded)));
        let content = node()
            .with_node(move |node| node.role(Role::Group))
            .name(detail_label)
            .with_node(move |node| node.part(disclosure.content()))
            .hidden(move || !content_visible.get())
            .on_key(move |event: KeyEvent| {
                if event.key() == Key::Escape {
                    expanded.set(false);
                    trigger_ref.restore_focus();
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .style(style().padding_left(22).padding_bottom(8))
            .child(detail.call(call));
        column((trigger, error, content))
    } else {
        column((summary, error)).style(style().padding_x(6))
    };
    summary.with_node(move |node| node.role(Role::ListItem))
}
