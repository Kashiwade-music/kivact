//! Role: Catalog page of Trace: lines, collapsible detail and several steps.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "trace",
        name: "Trace",
        description: "処理の手順や思考の経過を折りたたんで表示する部品です。開閉は標準の kivact::ui::disclosure で、Enter、Space、クリックで切り替わります。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("trace.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "lines",
                "Lines",
                "trace_line にアイコンと所要時間（description）を添えて並べます。",
                lines,
            ),
            Example::new(
                "collapsible",
                "Collapsible",
                "trace_collapsible に開閉の束縛、行、開くたびに内容を作る関数を渡します。",
                collapsible,
            ),
            Example::new(
                "steps",
                "Steps",
                "行と折りたたみを 1 つの trace にまとめます。",
                steps,
            ),
        ],
    }
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
fn steps() -> impl IntoView {
    // xtask-allow(state-fields): independent: each collapsible of the demo opens separately
    let first = signal(true);
    // xtask-allow(state-fields): independent: each collapsible of the demo opens separately
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
