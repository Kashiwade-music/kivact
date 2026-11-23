//! Catalog page of Code Block: title, line numbers, small size, scrolling.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("code-block", "Code Block", "Components", include_str!("code_block.rs"))
        .description("A code display with a title, a copy action, line numbers and a footer. Rust is colored from the core Rust tokens (kivact::ui::rust_tokens). The code is a read-only text box, so you can select part of it and copy it with Ctrl+C. The copy button writes to the clipboard through the core. Every code on this catalog is shown with this component.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "reactive",
            "Reactive source and theme",
            "The source is parsed only when it changes, and the colors follow the current theme.",
            reactive,
        ))
        .example(Example::new(
            "basic",
            "Title and copy",
            "The Rust tokens are computed once with rust_tokens, and highlighter passes them to the highlight callback. A press changes the icon to a check mark for 2 seconds.",
            basic,
        ))
        .example(Example::new(
            "line-numbers",
            "Line numbers",
            "line_numbers(true) shows the line numbers on the left.",
            line_numbers,
        ))
        .example(Example::new(
            "small",
            "Small and footer",
            "Pass CodeBlockSize::Sm as the variant for 12 px text. footer adds a note below.",
            small,
        ))
        .example(Example::new(
            "scroll",
            "Max height",
            "Code taller than max_height scrolls vertically. A long line scrolls horizontally too.",
            scroll_code,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let source = "fn main() {\n    println!(\"Hello, Kivact!\");\n}";
    kivact::ui::code_block(source.to_string())
        .highlight(kivact::ui::highlighter(
            kivact::ui::rust_tokens(source),
            |tokens, text, index| basic::rust_lines(tokens, text, index),
        ))
        .title("main.rs")
        .style(style().width(pct(100)))
}
// end example

// example: line-numbers
fn line_numbers() -> impl IntoView {
    let source = "let count = signal(0);\n// Increment on press.\ncount.update(|n| *n += 1);";
    kivact::ui::code_block(source.to_string())
        .copy_button(false)
        .highlight(kivact::ui::highlighter(
            kivact::ui::rust_tokens(source),
            |tokens, text, index| basic::rust_lines(tokens, text, index),
        ))
        .line_numbers(true)
        .style(style().width(pct(100)))
}
// end example

// example: small
fn small() -> impl IntoView {
    kivact::ui::code_block("cargo add kivact\ncargo run".to_string())
        .copy_button(false)
        .variant(basic::CodeBlockSize::Sm)
        .title("Terminal")
        .footer(basic::muted("Run both commands from your project root."))
        .style(style().width(pct(100)))
}
// end example

// example: scroll
fn scroll_code() -> impl IntoView {
    let lines = (1..=30)
        .map(|n| {
            format!("println!(\"line {n}: a rather long line of output to scroll sideways\");")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let tokens = kivact::ui::rust_tokens(&lines);
    kivact::ui::code_block(lines)
        .copy_button(false)
        .highlight(kivact::ui::highlighter(tokens, |tokens, text, index| {
            basic::rust_lines(tokens, text, index)
        }))
        .max_height(120.0)
        .style(style().width(pct(100)))
}
// end example

// example: reactive
fn reactive() -> impl IntoView {
    let source = signal("fn main() {}".to_string());
    let tokens = memo(move || kivact::ui::rust_tokens(&source.get()));
    column((
        kivact::ui::text_area(source).label("Source"),
        kivact::ui::code_block(source)
            .title("Editable source preview")
            .highlight(kivact::ui::highlighter(tokens, |tokens, text, index| {
                basic::rust_lines(tokens, text, index)
            })),
    ))
    .style(style().gap(12).width(pct(100)))
}
// end example
