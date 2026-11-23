//! Catalog page of Typography: headings, prose, lists, inline code and small text.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("typography", "Typography", "Components", include_str!("typography.rs"))
        .description("Text formatting: headings, paragraphs, quotes, lists and inline code. Unlike the 12 px Lyra components, the body is set at 16 px. The caller sets the spacing between blocks with gap.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "headings",
            "Headings",
            "Headings h1 to h4. Each has Role::Heading and its level in the hierarchy.",
            headings,
        ))
        .example(Example::new(
            "prose",
            "Prose",
            "A paragraph and a quote. A quote has a 2 px line on the left and is shown in italics.",
            prose,
        ))
        .example(Example::new(
            "list",
            "List",
            "bullet_list makes a bulleted list of strings.",
            list,
        ))
        .example(Example::new(
            "inline-code",
            "Inline code",
            "inline_code is in a monospace font and is drawn on a faint background.",
            inline_code,
        ))
        .example(Example::new(
            "text-styles",
            "Lead, large, small and muted",
            "A lead paragraph, large text, small text and muted text.",
            text_styles,
        ))
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
