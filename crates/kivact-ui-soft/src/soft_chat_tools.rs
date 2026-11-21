//! Role: Chat tools, token display and grapheme-safe streaming text.

use kivact::state::Signal;
use std::rc::Rc;
use std::time::Duration;

use crate::color as c;
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, MEDIUM};
use crate::radius::RADIUS_ELEMENT;
use crate::type_scale as t;
use crate::{SpinnerShade, SpinnerVariant};
use kivact::component;
use kivact::custom::EditState;
use kivact::i18n::{message, message_with};
use kivact::interaction::use_disclosure;
use kivact::motion::Transition;
use kivact::state::IntoValue;
use kivact::state::Machine;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::FlexDirection;
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

/// A tool call's detail content, built for each record of an expanded row.
type ToolDetails = Rc<dyn Fn(Value<ToolCall>) -> View>;

/// A labelled collection of tool calls with keyboard-operable group and detail disclosures.
///
/// `label` names the group's trigger and its list (not the root). `expanded` binds the group
/// disclosure to application-owned expansion state, and `detail` renders additional content for
/// an expanded tool-call row.
#[component]
pub fn chat_tool_calls(
    items: Value<Vec<ToolCall>>,
    label: Option<Value<String>>,
    expanded: Option<Signal<bool>>,
    detail: Option<Rc<dyn Fn(Value<ToolCall>) -> View>>,
) -> impl Element {
    let calls = items;
    let open = expanded.unwrap_or_else(|| signal(false));
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
    node()
        .hidden(move || calls.with(Vec::is_empty))
        .with_node(move |node| {
            node.base_style(
                style()
                    .flex_direction(FlexDirection::Column)
                    .gap(2)
                    .margin_top(8)
                    .font_size(12),
            )
        })
        .child((trigger, items))
}

fn disclosure_style() -> kivact::style::Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(6)
        .padding_x(6)
        .min_height(24)
        .border_radius(RADIUS_ELEMENT)
        .color(c::TEXT_SECONDARY)
        .hover(style().background(c::OVERLAY_HOVER))
        .focus_visible(style().outline(m::FOCUS_RING_WIDTH, c::FOCUS_RING))
}

fn tool_summary(call: Value<ToolCall>, visible: Value<bool>) -> (Node, Value<String>) {
    let (pending, running, result, status_label) = tool_status(call, visible);
    let (label, target, duration) = tool_labels(call);
    let (additions, deletions) = tool_diff(call);
    (
        row((
            pending, running, result, label, target, additions, deletions, duration,
        ))
        .style(
            style()
                .gap(6)
                .min_height(24)
                .min_width(0)
                .align_items(Align::Center)
                .color(c::TEXT_SECONDARY),
        ),
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
                .variant(move || SpinnerVariant {
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
        style().flex_shrink(0).single_line().color(
            if matches!(status.get(), ToolStatus::Error { .. }) {
                c::TEXT_ERROR
            } else {
                c::TEXT_SUCCESS
            },
        )
    });
    (pending, running, result, status_label)
}

fn tool_labels(call: Value<ToolCall>) -> (Node, Node, Node) {
    let label = text(move || call.with(|call| call.name.clone())).style(
        style()
            .font_family(t::FAMILY_CODE)
            .font_weight(500)
            .min_width(24)
            .single_line(),
    );
    let target = text(move || call.with(|call| call.target.clone().unwrap_or_default()))
        .hidden(move || call.with(|call| call.target.is_none()))
        .style(style().flex_shrink(10).min_width(0).single_line());
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
    .style(style().single_line().flex_shrink(0));
    (label, target, duration)
}

fn tool_diff(call: Value<ToolCall>) -> (Node, Node) {
    let additions = text(move || call.with(|call| format!("+{}", call.additions)))
        .hidden(move || call.with(|call| call.additions == 0))
        .style(style().color(c::TEXT_SUCCESS).single_line().flex_shrink(0));
    let deletions = text(move || call.with(|call| format!("−{}", call.deletions)))
        .hidden(move || call.with(|call| call.deletions == 0))
        .style(style().color(c::TEXT_ERROR).single_line().flex_shrink(0));
    (additions, deletions)
}

fn tool_chevron(open: impl IntoValue<bool>) -> Node {
    let open = open.into_value();
    icon("chevron-down").style(move || {
        style()
            .size(14, 14)
            .flex_shrink(0)
            .color(c::ICON_SECONDARY)
            .rotate(if open.get() { 180 } else { 0 })
            .transition(TransitionProp::All, Transition::tween(125.0, EASE_STANDARD))
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Semantic color treatment of a substituted chat token.
pub(crate) enum ChatTokenTone {
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
pub(crate) struct ChatToken {
    /// Nonempty source substring to replace when tokenizing text.
    pub value: String,
    /// Visible replacement label.
    pub label: String,
    /// Semantic color treatment of the replacement.
    pub tone: ChatTokenTone,
}

impl ChatToken {
    /// Create a neutral token replacing the supplied source substring with a label.
    pub(crate) fn badge(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            tone: ChatTokenTone::Neutral,
        }
    }
    /// Set the semantic color treatment of this token.
    pub(crate) fn tone(mut self, tone: ChatTokenTone) -> Self {
        self.tone = tone;
        self
    }
}

fn token_colors(tone: ChatTokenTone) -> (kivact::style::Color, kivact::style::Color) {
    match tone {
        ChatTokenTone::Neutral => (c::NEUTRAL, c::TEXT_PRIMARY),
        ChatTokenTone::Info => (c::INFO_MUTED, c::ON_INFO_MUTED),
        ChatTokenTone::Success => (c::SUCCESS_MUTED, c::ON_SUCCESS_MUTED),
        ChatTokenTone::Warning => (c::WARNING_MUTED, c::ON_WARNING_MUTED),
        ChatTokenTone::Error => (c::ERROR_MUTED, c::ON_ERROR_MUTED),
    }
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
                .font_weight(500)
                .background(fill)
                .color(ink),
        );
        rest = &rest[index + token.value.len()..];
    }
    spans
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Reveal pacing used while streamed text is arriving.
pub(crate) enum StreamSpeed {
    #[default]
    /// Reveal text in the standard paced batches.
    Natural,
    /// Use shorter ticks and smaller reveal batches.
    Fast,
    /// Show the complete target immediately.
    Instant,
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
    open: Signal<bool>,
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
            summary.style(style().flex_grow(1).min_width(0)),
            icon("wrench").style(style().size(12, 12).flex_shrink(0)),
            text(move || calls.with(|calls| calls.len().to_string())).style(style().flex_shrink(0)),
        ))
        .style(
            style()
                .gap(4)
                .align_items(Align::Center)
                .flex_grow(1)
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
                    .flex_grow(1)
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
                .style(style().gap(2).flex_shrink(0)),
        )
        .with_node(move |node| node.role(Role::Group))
        .with_node(move |node| node.part(disclosure.content()))
        .style(
            style()
                .enter_size(kivact::view::Orientation::Vertical)
                .exit_size(kivact::view::Orientation::Vertical)
                .transition(
                    TransitionProp::Size,
                    Transition::tween(MEDIUM as f64, EASE_STANDARD),
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
    .style(style().padding_left(22).color(c::TEXT_ERROR));
    let summary = if let Some(detail) = &detail {
        let expanded = signal(false);
        // The detail content is visible only while the group shows its rows.
        let content_visible = memo(move || expanded.get() && rows_visible.get());
        let disclosure = use_disclosure(expanded);
        let detail_label = message_with(
            "soft.chat.tool_calls.detail",
            "Details for {name}",
            move || [("name", call.with(|call| call.name.clone()))],
        );
        let trigger_ref = node_ref();
        let trigger = node()
            .disabled(move || !rows_visible.get())
            .with_node(move |node| node.part(disclosure.trigger()))
            .name(detail_label)
            .description(status)
            .node_ref(trigger_ref)
            .with_node(move |node| node.base_style(disclosure_style))
            .child((summary.style(style().flex_grow(1)), tool_chevron(expanded)));
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
            .child(detail(call));
        column((trigger, error, content))
    } else {
        column((summary, error)).style(style().padding_x(6))
    };
    summary.with_node(move |node| node.role(Role::ListItem))
}
