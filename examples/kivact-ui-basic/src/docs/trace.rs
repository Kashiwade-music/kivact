//! Role: Catalog page of Trace: lines, collapsible detail and several steps.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "trace",
        name: "Trace",
        description: "処理の手順や思考の経過を折りたたんで表示する部品です。開閉は kivact::interaction::use_disclosure で、Enter、Space、クリックで切り替わります。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("trace.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "lines",
                "Lines",
                "行にアイコンと所要時間を添えて並べます。",
                lines,
            ),
            Example::new(
                "collapsible",
                "Collapsible",
                "disclosure に開閉の束縛と内容を渡し、attach でアイコン付きの行をトリガーにします。",
                collapsible,
            ),
            Example::new(
                "steps",
                "Steps",
                "行と折りたたみを 1 つの column にまとめます。",
                steps,
            ),
        ],
    }
}

// example: lines
fn lines() -> impl IntoView {
    column((
        row((
            icon("search").style(style().size(14, 14)),
            text("Searched the web").style(style().single_line()),
            text("1.2s").style(move || {
                style().shrink(0).tabular_nums().color(
                    kivact::style::use_tokens::<basic::Tokens>()
                        .muted_foreground
                        .with_alpha(0.7),
                )
            }),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        row((
            icon("file-text").style(style().size(14, 14)),
            text("Read 3 files"),
        ))
        .style(style().gap(8).align_items(Align::Center)),
    ))
    .style(
        style()
            .width(420)
            .min_width(0)
            .gap(8)
            .font_size(12)
            .color(basic::ColorName::MutedForeground),
    )
}
// end example

// example: collapsible
fn collapsible() -> impl IntoView {
    let open = signal(false);
    kivact::ui::disclosure("Thought for 4s", open)
        .variant(basic::DisclosureVariant::Trace)
        .animated(false)
        .content(|| text("First I compared the two layouts, then picked the simpler one."))
        .attach(
            row((
                icon("sparkles").style(style().size(14, 14)),
                text("Thought for 4s"),
                icon("chevron-right")
                    .style(move || style().size(14, 14).rotate(if open.get() { 90 } else { 0 })),
            ))
            .name("Thought for 4s")
            .style(
                style()
                    .gap(4)
                    .align_items(Align::Center)
                    .align_self(Align::Start)
                    .font_size(12)
                    .color(basic::ColorName::MutedForeground)
                    .hover(style().color(basic::ColorName::Foreground)),
            ),
        )
        .style(style().width(420))
}
// end example

// example: steps
fn steps() -> impl IntoView {
    let first = signal(true);
    let second = signal(false);
    column((
        kivact::ui::disclosure("Ran the tests", first)
            .variant(basic::DisclosureVariant::Trace)
            .animated(false)
            .content(|| text("42 passed, 0 failed."))
            .attach(
                row((
                    icon("terminal").style(style().size(14, 14)),
                    text("Ran the tests"),
                    icon("chevron-right").style(move || {
                        style()
                            .size(14, 14)
                            .rotate(if first.get() { 90 } else { 0 })
                    }),
                ))
                .name("Ran the tests")
                .style(
                    style()
                        .gap(4)
                        .align_items(Align::Center)
                        .align_self(Align::Start)
                        .font_size(12)
                        .color(basic::ColorName::MutedForeground)
                        .hover(style().color(basic::ColorName::Foreground)),
                ),
            ),
        kivact::ui::disclosure("Edited 2 files", second)
            .variant(basic::DisclosureVariant::Trace)
            .animated(false)
            .content(|| text("src/lib.rs and src/main.rs"))
            .attach(
                row((
                    icon("pencil").style(style().size(14, 14)),
                    text("Edited 2 files"),
                    icon("chevron-right").style(move || {
                        style()
                            .size(14, 14)
                            .rotate(if second.get() { 90 } else { 0 })
                    }),
                ))
                .name("Edited 2 files")
                .style(
                    style()
                        .gap(4)
                        .align_items(Align::Center)
                        .align_self(Align::Start)
                        .font_size(12)
                        .color(basic::ColorName::MutedForeground)
                        .hover(style().color(basic::ColorName::Foreground)),
                ),
            ),
    ))
    .style(style().width(420).min_width(0).gap(8))
}
// end example
