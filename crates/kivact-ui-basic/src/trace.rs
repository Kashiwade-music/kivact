//! Role: The Lyra trace: muted activity lines with a collapsible detail.

use crate::Color;
use crate::parts::sized_icon;
use crate::theme_access::tokens;
use kivact::interaction::use_disclosure;
use kivact::view::icon;
use kivact::view::{Node, NodeBuilder, column, node, row, text};

use kivact::state::IntoText;
use kivact::state::Value;
use kivact::style::{Align, Cursor, Direction, pct, style};
use kivact::view::{IntoView, View};

/// A stack of trace lines and collapsibles with 8 px between them.
#[track_caller]
pub fn trace(children: impl IntoView) -> impl kivact::view::Component {
    column(children).style(style().width(pct(100)).min_width(0).gap(8))
}

/// One line of a [`trace`]: an optional icon, a title and a description.
/// Created by [`trace_line`].
pub struct TraceLine {
    root: Node,
    title: Value<String>,
    icon: Option<View>,
    description: Option<Value<String>>,
}

/// A trace line titled `title`, in muted 12 px text.
///
/// ```no_run
/// use kivact::view::{icon};
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn steps() -> impl IntoView {
///     basic::trace((
///         basic::trace_line("Searched the web").icon(icon("search")).description("1.2s"),
///         basic::trace_line("Read 3 files").icon(icon("file-text")),
///     ))
/// }
/// ```
#[track_caller]
pub fn trace_line(title: impl IntoText) -> TraceLine {
    let title = title.into_text_value();
    TraceLine {
        root: node().name(title),
        title,
        icon: None,
        description: None,
    }
}

impl TraceLine {
    /// Show `icon` before the title at 14 px.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Show `text` after the title, fainter (a duration or a count).
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// The row with its children; the caller adds behavior and state. A line
    /// that opens a collapsible is its `trigger`.
    fn build(self, trigger: bool) -> Node {
        let title = self.title;
        self.root
            .base_style(move || {
                let p = tokens();
                let s = style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(8)
                    .max_width(pct(100))
                    .min_width(0)
                    .font_size(12)
                    .color(p.muted_foreground);
                if trigger {
                    s.gap(4)
                        .cursor(Cursor::Pointer)
                        .hover(style().color(p.foreground))
                } else {
                    s
                }
            })
            .child(self.icon.map(|icon| sized_icon(icon, 14.0)))
            .child(text(title).style(style().single_line()))
            .child(self.description.map(|description| {
                text(description).style(move || {
                    style()
                        .shrink(0)
                        .tabular_nums()
                        .color(tokens().muted_foreground.with_alpha(0.7))
                })
            }))
    }
}

impl NodeBuilder for TraceLine {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TraceLine {
    fn into_view(self) -> View {
        self.build(false).into_view()
    }
}

/// A trace line that opens `content` below it (trace.tsx `TraceCollapsible`).
///
/// The line is the trigger: Enter, Space or a click writes to `open`, a
/// chevron turns down while open, and the content hangs from a left rule.
/// It uses [`kivact::interaction::use_disclosure`].
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn details() -> impl IntoView {
///     let open = signal(false);
///     basic::trace_collapsible(open, basic::trace_line("Thought for 4s"), "The reasoning goes here.")
/// }
/// ```
#[track_caller]
pub fn trace_collapsible(
    open: impl kivact::state::IntoBinding<bool>,
    line: TraceLine,
    content: impl IntoView,
) -> impl kivact::view::Component {
    let section = use_disclosure(open);
    let shown = section.is_open();
    let line = line.build(true);
    let trigger_part = section.trigger();
    let trigger = line.with_node(|root| root.part(trigger_part)).child(
        node()
            .style(move || {
                style()
                    .size(14, 14)
                    .shrink(0)
                    .rotate(if shown.get() { 90.0 } else { 0.0 })
            })
            .child(sized_icon(icon("chevron-right"), 14.0)),
    );
    let body = column(content)
        .with_node(|root| root.part(section.content()))
        .style(
            style()
                .min_width(0)
                .margin_top(8)
                .margin_bottom(4)
                .margin_left(6)
                .padding_left(15)
                .padding_right(4)
                .border_left(1, Color::Border)
                .line_height(1.625)
                .color(Color::MutedForeground)
                .font_size(12),
        );
    column((row(trigger).style(style().align_self(Align::Start)), body))
        .style(style().width(pct(100)).min_width(0))
}
