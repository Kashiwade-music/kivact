//! Role: Render one documentation page: header, usage and example panels with code.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::view::Scroll;
use kivact_ui_basic as basic;

/// The scrolling page of `page`; a new page starts at the top.
pub(crate) fn page_view(page: &'static Page) -> Scroll {
    scroll(column(content(page)).style(style().align_items(Align::Center)))
        .test_id("catalog-page")
        .style(style().grow(1).min_height(0))
}

fn content(page: &'static Page) -> Node {
    let header = column((
        breadcrumb(page.name),
        basic::h1(page.name).test_id("catalog-title"),
        basic::lead(page.description),
    ))
    .style(style().gap(12));
    let usage = column((
        basic::h2("使い方"),
        code_block(page.usage.to_string(), "catalog-usage-code".to_string()),
    ))
    .style(style().gap(16));
    let examples = page
        .examples
        .iter()
        .map(|example| example_section(page, example))
        .collect::<Vec<_>>();
    column((header, usage, examples)).style(
        style()
            .width(pct(100))
            .max_width(768)
            .padding_x(48)
            .padding_y(40)
            .gap(40),
    )
}

fn breadcrumb(name: &'static str) -> impl kivact::view::Component {
    basic::breadcrumb_content((
        basic::breadcrumb_link("コンポーネント"),
        basic::breadcrumb_separator(),
        basic::breadcrumb_page(name),
    ))
    .style(style().font_size(14))
}

/// An example's title: 20 px semibold on a 28 px line (shadcn docs' `h3`).
fn example_title(title: &'static str) -> Node {
    text(title).role(Role::Heading).level(3).style(
        style()
            .font_size(20)
            .line_height(1.4)
            .weight(600)
            .color(basic::Color::Foreground),
    )
}

/// Explanatory prose: 14 px on a 24 px line.
fn prose(content: &'static str) -> Node {
    text(content).style(
        style()
            .font_size(14)
            .line_height(24.0 / 14.0)
            .color(basic::Color::Foreground),
    )
}

/// A heading, its explanation, the live preview and the code that built it.
fn example_section(page: &'static Page, example: &'static Example) -> Node {
    let prefix = format!("example-{}-{}", page.slug, example.id);
    let along = match page.align {
        PreviewAlign::Center => Justify::Center,
        PreviewAlign::Start => Justify::Start,
        PreviewAlign::End => Justify::End,
    };
    let preview = column(example.view())
        .test_id(format!("{prefix}-preview"))
        .style(
            style()
                .min_height(200)
                .padding(40)
                .align_items(Align::Center)
                .justify_content(along)
                .border(1, basic::Color::Border),
        );
    let code = code_block(page.snippet(example.id), format!("{prefix}-code"))
        .style(style().margin_top(-1));
    column((
        example_title(example.title),
        prose(example.description),
        column((preview, code)),
    ))
    .style(style().gap(12))
}

/// A highlighted "Rust" block whose copy button writes `snippet` to the clipboard.
fn code_block(snippet: String, test_id: String) -> basic::CodeBlock {
    basic::code_block(basic::rust_lines(&snippet))
        .title("Rust")
        .on_copy(move || {
            spawn(kivact::platform::copy_text(snippet.clone()));
        })
        .test_id(test_id)
}
