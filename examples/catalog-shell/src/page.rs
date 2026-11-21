//! Role: Render one documentation page: header, usage and example panels with code.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::style::StyledRange;
use kivact::ui::SyntaxKind;
use kivact::view::Node;
use kivact::view::Scroll;
use std::ops::Range;

/// The scrolling page of `page`; a new page starts at the top.
pub(crate) fn page_view(page: &Page) -> Scroll {
    scroll(column(content(page)).style(style().align_items(Align::Center)))
        .test_id("catalog-page")
        .style(style().flex_grow(1).min_height(0))
}

fn content(page: &Page) -> Node {
    let header = column((
        breadcrumb(page.name),
        kivact::ui::heading(page.name, 1).test_id("catalog-title"),
        lead(page.description),
    ))
    .style(style().gap(12));
    let usage = (!page.usage.is_empty()).then(|| {
        column((
            kivact::ui::heading("Usage", 2),
            code_block(page.usage.to_string(), "catalog-usage-code".to_string()),
        ))
        .style(style().gap(16))
    });
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

fn breadcrumb(name: &'static str) -> impl IntoView {
    kivact::ui::breadcrumb(vec![
        kivact::interaction::ListItem::new("root", "Components"),
        kivact::interaction::ListItem::new("current", name),
    ])
    .separator(|| icon("chevron-right").style(style().size(14, 14)))
    .style(style().font_size(14))
}

/// The page's explanation: 16 px in the muted text color.
fn lead(content: &'static str) -> Node {
    text(content).style(style().font_size(16).color(color::TEXT_MUTED))
}

/// An example's title: 20 px semibold on a 28 px line (shadcn docs' `h3`).
fn example_title(title: &'static str) -> Node {
    text(title).role(Role::Heading).level(3).style(
        style()
            .font_size(20)
            .line_height(1.4)
            .font_weight(600)
            .color(color::TEXT),
    )
}

/// Explanatory prose: 14 px on a 24 px line.
fn prose(content: &'static str) -> Node {
    text(content).style(
        style()
            .font_size(14)
            .line_height(24.0 / 14.0)
            .color(color::TEXT),
    )
}

/// An example's heading, its explanation, the live preview and the code that built it.
fn example_section(page: &Page, example: &Example) -> Node {
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
                .border(1, color::BORDER),
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

/// A "Rust" block whose copy button writes `snippet` to the clipboard, its
/// tokens colored from the core's Rust tokens (`kivact::ui::rust_tokens`).
fn code_block(snippet: String, test_id: String) -> kivact::ui::CodeBlock {
    let tokens = kivact::ui::rust_tokens(&snippet);
    kivact::ui::code_block(snippet)
        .highlight(kivact::ui::highlighter(tokens, |tokens, text, index| {
            rust_line(tokens, text, index)
        }))
        .title("Rust")
        .test_id(test_id)
}

/// The colored ranges of line `index` of `text`, and the index of the next line.
fn rust_line(
    tokens: &[Vec<(Range<usize>, SyntaxKind)>],
    text: &str,
    index: &usize,
) -> (Vec<StyledRange>, usize) {
    let line = tokens.get(*index).map_or(&[][..], Vec::as_slice);
    (kivact::ui::syntax_ranges(text, line), index + 1)
}
