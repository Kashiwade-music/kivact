//! Role: Catalog page of Typography: headings, prose, lists, inline code and small text.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "typography",
        name: "Typography",
        description: "見出し、段落、引用、リスト、インラインのコードなど、文章の書式です。Lyra の 12 px の部品とは別に、本文は 16 px で組みます。ブロックの間隔は呼び出し側の gap で決めます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("typography.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "headings",
                "Headings",
                "h1 から h4 までの見出しです。それぞれ Role::Heading と見出しの階層を持ちます。",
                headings,
            ),
            Example::new(
                "prose",
                "Prose",
                "段落と引用です。引用は左に 2 px の線を引き、斜体で表示します。",
                prose,
            ),
            Example::new(
                "list",
                "List",
                "bullet_list は文字列の一覧を箇条書きにします。",
                list,
            ),
            Example::new(
                "inline-code",
                "Inline code",
                "inline_code は等幅のフォントで、薄い背景を敷いて表示します。",
                inline_code,
            ),
            Example::new(
                "text-styles",
                "Lead, large, small and muted",
                "リード文、大きな文字、小さな文字、補足の文字です。",
                text_styles,
            ),
        ],
    }
}

// example: headings
fn headings() -> impl IntoView {
    column((
        kivact::ui::heading("Taxing Laughter", 1),
        kivact::ui::heading("The People of the Kingdom", 2),
        kivact::ui::heading("The Joke Tax", 3),
        kivact::ui::heading("People stopped telling jokes", 4),
    ))
    .style(style().gap(16).width(pct(100)))
}
// end example

// example: prose
fn prose() -> impl IntoView {
    column((
        basic::paragraph("The king saw how happy his subjects were and repealed the joke tax."),
        basic::blockquote(
            "After all, everyone enjoys a good joke, so it's only fair to pay for it.",
        ),
    ))
    .style(style().gap(24).width(pct(100)))
}
// end example

// example: list
fn list() -> impl IntoView {
    column((
        basic::paragraph(
            "The king thought long and hard, and finally came up with a brilliant plan:",
        ),
        basic::bullet_list(vec![
            "1st level of Jokes: 5 gold coins".to_string(),
            "2nd level of Jokes: 10 gold coins".to_string(),
            "3rd level of Jokes: 20 gold coins".to_string(),
        ]),
    ))
    .style(style().gap(16).width(pct(100)))
}
// end example

// example: inline-code
fn inline_code() -> impl IntoView {
    row((
        basic::paragraph("Install the crate with"),
        basic::inline_code("cargo add kivact"),
        basic::paragraph("and run it."),
    ))
    .style(style().gap(6).align_items(kivact::style::Align::Center))
}
// end example

// example: text-styles
fn text_styles() -> impl IntoView {
    column((
        basic::lead("A modal dialog that interrupts the user with important content."),
        basic::large("Are you absolutely sure?"),
        basic::small("Email address"),
        basic::muted("Enter your email address."),
    ))
    .style(style().gap(16).width(pct(100)))
}
// end example
