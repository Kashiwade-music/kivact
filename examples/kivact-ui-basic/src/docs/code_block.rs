//! Role: Catalog page of Code Block: title, line numbers, small size, scrolling.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "code-block",
        name: "Code Block",
        description: "題名、コピーの操作、行番号、フッターを持つコードの表示部品です。Rust は rust_lines で色分けでき、ほかの言語は呼び出し側が CodeLine と CodeSpan で色を渡します。コードは読み取り専用の text box で、一部を選んで Ctrl+C でコピーできます。コピーのボタンの書き込みはアプリが行います。このカタログの各コードも、この部品で表示しています。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("code_block.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Title and copy",
                "rust_lines で Rust を色分けし、title で見出しを付け、on_copy に押されたときの処理を渡します。押すと 2 秒間、アイコンがチェックに変わります。",
                basic,
            ),
            Example::new(
                "line-numbers",
                "Line numbers",
                "line_numbers(true) で、左に行番号を表示します。",
                line_numbers,
            ),
            Example::new(
                "small",
                "Small and footer",
                "size に CodeBlockSize::Sm を渡すと 12 px の文字になります。footer で下に補足を添えられます。",
                small,
            ),
            Example::new(
                "scroll",
                "Max height",
                "max_height を超えるコードは縦にスクロールします。横に長い行も横にスクロールします。",
                scroll_code,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let source = "fn main() {\n    println!(\"Hello, Kivact!\");\n}";
    basic::code_block(basic::rust_lines(source))
        .title("main.rs")
        .on_copy(move || {
            spawn(kivact::platform::copy_text(source));
        })
        .style(style().width(pct(100)))
}
// end example

// example: line-numbers
fn line_numbers() -> impl IntoView {
    use basic::{CodeLine, CodeSpan, CodeTone};
    let lines = vec![
        CodeLine::new(vec![
            CodeSpan::new("let", CodeTone::Keyword),
            CodeSpan::new(" count = ", CodeTone::Plain),
            CodeSpan::new("signal", CodeTone::Function),
            CodeSpan::new("(0);", CodeTone::Punctuation),
        ]),
        CodeLine::new(vec![CodeSpan::new(
            "// Increment on press.",
            CodeTone::Comment,
        )]),
        CodeLine::new(vec![
            CodeSpan::new("count.", CodeTone::Plain),
            CodeSpan::new("update", CodeTone::Function),
            CodeSpan::new("(|n| *n += 1);", CodeTone::Punctuation),
        ]),
    ];
    basic::code_block(lines)
        .line_numbers(true)
        .style(style().width(pct(100)))
}
// end example

// example: small
fn small() -> impl IntoView {
    basic::code_block(basic::code_lines("cargo add kivact\ncargo run"))
        .size(basic::CodeBlockSize::Sm)
        .title("Terminal")
        .footer(move || basic::muted("Run both commands from your project root."))
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
    basic::code_block(basic::rust_lines(&lines))
        .max_height(120.0)
        .style(style().width(pct(100)))
}
// end example
