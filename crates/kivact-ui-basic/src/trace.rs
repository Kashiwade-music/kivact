//! Role: The Lyra trace: muted activity lines with a collapsible detail.
use crate::Tokens;
use crate::parts::sized_icon;
use kivact::state::{IntoBinding, IntoText, Value};
use kivact::style::{Align, Cursor, Direction, Style, Theme, pct, style, use_tokens};
use kivact::view::{Element, IntoView, IntoViewFn, Node, View, column, icon, node, text};

/// A stack of trace lines and collapsibles with 8 px between them.
#[track_caller]
pub fn trace(children: impl IntoView) -> Node {
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

    /// The row with its children. A line that opens a collapsible is its
    /// trigger: it hugs its content, uses a pointer and brightens on hover.
    fn build(self, trigger: bool) -> Node {
        self.root
            .base_style(move || {
                let p = use_tokens::<Tokens>();
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
                        .align_self(Align::Start)
                        .cursor(Cursor::Pointer)
                        .hover(style().color(p.foreground))
                } else {
                    s
                }
            })
            .child(self.icon.map(|icon| sized_icon(icon, 14.0)))
            .child(text(self.title).style(style().single_line()))
            .child(self.description.map(|description| {
                text(description).style(move || {
                    style()
                        .shrink(0)
                        .tabular_nums()
                        .color(use_tokens::<Tokens>().muted_foreground.with_alpha(0.7))
                })
            }))
    }
}

impl Element for TraceLine {
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
/// The line is the trigger of a standard `kivact::ui::disclosure`: Enter,
/// Space or a click writes to `open`, a chevron turns down while open, and
/// the content, built each time the section opens, hangs from a left rule.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn details() -> impl IntoView {
///     let open = signal(false);
///     basic::trace_collapsible(open, basic::trace_line("Thought for 4s"), || {
///         text("The reasoning goes here.")
///     })
/// }
/// ```
#[track_caller]
pub fn trace_collapsible<V: IntoView>(
    open: impl IntoBinding<bool>,
    line: TraceLine,
    content: impl IntoViewFn<V>,
) -> impl Element {
    let open = open.into_binding();
    let title = line.title;
    let chevron = node()
        .style(move || {
            style()
                .size(14, 14)
                .shrink(0)
                .rotate(if open.get() { 90 } else { 0 })
        })
        .child(sized_icon(icon("chevron-right"), 14.0));
    kivact::ui::disclosure(title, open)
        .variant(crate::DisclosureVariant::Trace)
        .animated(false)
        .content(content)
        .attach(line.build(true).child(chevron))
        .style(style().width(pct(100)).min_width(0))
}

/// The detail of a [`trace_collapsible`], the content part of a disclosure
/// in [`DisclosureVariant::Trace`](crate::DisclosureVariant::Trace).
pub(crate) fn content(t: &Theme) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Column)
        .min_width(0)
        .margin_top(8)
        .margin_bottom(4)
        .margin_left(6)
        .padding_left(15)
        .padding_right(4)
        .border_left(1, p.border)
        .line_height(1.625)
        .color(p.muted_foreground)
        .font_size(12)
}
