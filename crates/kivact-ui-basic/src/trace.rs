//! Role: The Lyra trace: muted activity lines with a collapsible detail.
use crate::color::{FOREGROUND, MUTED_FOREGROUND};
use crate::recipe_parts::sized_icon;
use crate::variants::DisclosureVariant;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::{IntoValue, Signal, Value};
use kivact::style::{Align, Cursor, FlexDirection, pct, style};
use kivact::view::{Element, Node, View, ViewFn, column, icon, node, text};

/// A stack of trace lines and collapsibles with 8 px between them.
#[component]
pub fn trace(children: View) -> impl Element {
    column(children).style(style().width(pct(100)).min_width(0).gap(8))
}

/// The row of a trace line: an optional icon, the title and an optional
/// description. A line that opens a collapsible is its trigger: it hugs its
/// content, uses a pointer and brightens on hover.
fn line_row(
    title: Value<String>,
    icon: Option<View>,
    description: Option<Value<String>>,
    trigger: bool,
) -> Node {
    node()
        .name(title)
        .base_style(move || {
            let s = style()
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(8)
                .max_width(pct(100))
                .min_width(0)
                .font_size(12)
                .color(MUTED_FOREGROUND);
            if trigger {
                s.gap(4)
                    .align_self(Align::Start)
                    .cursor(Cursor::Pointer)
                    .hover(style().color(FOREGROUND))
            } else {
                s
            }
        })
        .child(icon.map(|icon| sized_icon(icon, 14.0)))
        .child(text(title).style(style().single_line()))
        .child(description.map(|description| {
            text(description).style(move || {
                style()
                    .flex_shrink(0.0)
                    .tabular_nums()
                    .color(MUTED_FOREGROUND.with_alpha(0.7))
            })
        }))
}

/// One line of a [`trace`]: an optional icon and a title in muted 12 px text,
/// with the element's `.description(..)` after it in fainter text (a duration
/// or a count).
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
#[component]
pub fn trace_line(
    title: Value<String>,
    icon: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    line_row(title, icon, caller.description(), false)
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
#[component]
pub fn trace_collapsible(open: Signal<bool>, mut line: TraceLine, content: ViewFn) -> impl Element {
    let description = line.node_mut().authored().description();
    let TraceLine {
        title,
        icon: line_icon,
        ..
    } = line;
    let title: Value<String> = IntoValue::into_value(title);
    let chevron = node()
        .style(move || {
            style()
                .size(14, 14)
                .flex_shrink(0.0)
                .rotate(if open.get() { 90 } else { 0 })
        })
        .child(sized_icon(icon("chevron-right"), 14.0));
    let trigger = line_row(title, line_icon, description, true).child(chevron);
    kivact::ui::disclosure(title, open)
        .variant(DisclosureVariant::Trace)
        .animated(false)
        .content(content)
        .attach(trigger)
        .style(style().width(pct(100)).min_width(0))
}
