//! Catalog page of Trace: lines, collapsible detail and several steps.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("trace", "Trace", "Components", include_str!("trace.rs"))
        .description("A component that shows the steps or the progress of thinking, folded away. It opens and closes with the standard kivact::ui::disclosure; Enter, Space and a click switch it.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "lines",
            "Lines",
            "trace_line rows, each with an icon and a duration (description), in a row.",
            lines,
        ))
        .example(Example::new(
            "collapsible",
            "Collapsible",
            "trace_collapsible takes the open binding, the lines, and a function that makes the content each time it opens.",
            collapsible,
        ))
        .example(Example::new(
            "steps",
            "Steps",
            "Put the lines and the folding together in one trace.",
            steps,
        ))
}

// example: lines
fn lines() -> impl IntoView {
    basic::trace((
        basic::trace_line("Searched the web")
            .icon(icon("search"))
            .description("1.2s"),
        basic::trace_line("Read 3 files").icon(icon("file-text")),
    ))
    .style(style().width(420))
}
// end example

// example: collapsible
fn collapsible() -> impl IntoView {
    let open = signal(false);
    basic::trace_collapsible(
        open,
        basic::trace_line("Thought for 4s").icon(icon("sparkles")),
        || text("First I compared the two layouts, then picked the simpler one."),
    )
    .style(style().width(420))
}
// end example

// example: steps
// xtask-allow(state): legacy: first, second; the open binding of the first trace collapsible, toggled only by its header; the second collapsible has its own binding, so both,
fn steps() -> impl IntoView {
    let first = signal(true);
    let second = signal(false);
    basic::trace((
        basic::trace_collapsible(
            first,
            basic::trace_line("Ran the tests").icon(icon("terminal")),
            || text("42 passed, 0 failed."),
        ),
        basic::trace_collapsible(
            second,
            basic::trace_line("Edited 2 files").icon(icon("pencil")),
            || text("src/lib.rs and src/main.rs"),
        ),
    ))
    .style(style().width(420))
}
// end example
