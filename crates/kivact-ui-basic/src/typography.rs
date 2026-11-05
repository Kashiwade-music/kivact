//! Role: shadcn/ui typography styles for headings, prose, quotes, code and lists.
//!
//! The styles of the shadcn/ui typography page. They are not part of the
//! Lyra preset's 12 px controls: prose uses 16 px body text. Vertical
//! spacing between blocks is the caller's (`column(..).style(style().gap(..))`).

use crate::ColorName;

use kivact::prelude::NodeBuilder;
use kivact::view::{Node, column, node, row, text};

use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, Direction, Style, style};
use kivact::view::{Role, for_index};

/// The monospace family of inline code and code blocks; the application registers the font.
pub(crate) const MONO: &str = "\"Geist Mono\", monospace";

/// A text node in the foreground color with `style` on top.
fn styled(value: impl IntoText, look: fn(&crate::Tokens) -> Style) -> Node {
    text(value).style(move || {
        let p = kivact::style::use_tokens::<crate::Tokens>();
        look(&p)
    })
}

/// A heading of `level` with the accessible name of its text.
fn heading(value: impl IntoText, level: u32, look: fn(&crate::Tokens) -> Style) -> Node {
    let value = value.into_text_value();
    styled(value, look)
        .with_node(|root| root.role(Role::Heading))
        .level(level)
        .name(value)
}

/// `scroll-m-20 text-4xl font-extrabold tracking-tight`: a level-1 heading.
#[track_caller]
pub fn h1(value: impl IntoText) -> impl kivact::view::Component {
    heading(value, 1, |p| {
        style()
            .font_size(36)
            .line_height(40.0 / 36.0)
            .weight(800)
            .letter_spacing(-0.9)
            .color(p.foreground)
    })
}

/// `border-b pb-2 text-3xl font-semibold tracking-tight`: a level-2 heading with a rule.
#[track_caller]
pub fn h2(value: impl IntoText) -> impl kivact::view::Component {
    heading(value, 2, |p| {
        style()
            .font_size(30)
            .line_height(36.0 / 30.0)
            .weight(600)
            .letter_spacing(-0.75)
            .padding_bottom(8)
            .border_bottom(1, p.border)
            .color(p.foreground)
    })
}

/// `text-2xl font-semibold tracking-tight`: a level-3 heading.
#[track_caller]
pub fn h3(value: impl IntoText) -> impl kivact::view::Component {
    heading(value, 3, |p| {
        style()
            .font_size(24)
            .line_height(32.0 / 24.0)
            .weight(600)
            .letter_spacing(-0.6)
            .color(p.foreground)
    })
}

/// `text-xl font-semibold tracking-tight`: a level-4 heading.
#[track_caller]
pub fn h4(value: impl IntoText) -> impl kivact::view::Component {
    heading(value, 4, |p| {
        style()
            .font_size(20)
            .line_height(28.0 / 20.0)
            .weight(600)
            .letter_spacing(-0.5)
            .color(p.foreground)
    })
}

/// `leading-7`: a 16 px paragraph.
#[track_caller]
pub fn paragraph(value: impl IntoText) -> impl kivact::view::Component {
    styled(value, |p| {
        style()
            .font_size(16)
            .line_height(28.0 / 16.0)
            .color(p.foreground)
    })
}

/// `text-xl text-muted-foreground`: an introductory paragraph.
#[track_caller]
pub fn lead(value: impl IntoText) -> impl kivact::view::Component {
    styled(value, |p| {
        style()
            .font_size(20)
            .line_height(28.0 / 20.0)
            .color(p.muted_foreground)
    })
}

/// `text-lg font-semibold`: emphasized text.
#[track_caller]
pub fn large(value: impl IntoText) -> impl kivact::view::Component {
    styled(value, |p| {
        style()
            .font_size(18)
            .line_height(28.0 / 18.0)
            .weight(600)
            .color(p.foreground)
    })
}

/// `text-sm leading-none font-medium`: small print.
#[track_caller]
pub fn small(value: impl IntoText) -> impl kivact::view::Component {
    styled(value, |p| {
        style()
            .font_size(14)
            .line_height(1.0)
            .weight(500)
            .color(p.foreground)
    })
}

/// `text-sm text-muted-foreground`: secondary text.
#[track_caller]
pub fn muted(value: impl IntoText) -> impl kivact::view::Component {
    styled(value, |p| {
        style()
            .font_size(14)
            .line_height(20.0 / 14.0)
            .color(p.muted_foreground)
    })
}

/// `border-l-2 pl-6 italic`: a quotation.
#[track_caller]
pub fn blockquote(value: impl IntoText) -> impl kivact::view::Component {
    styled(value, |p| {
        style()
            .font_size(16)
            .line_height(28.0 / 16.0)
            .italic(true)
            .padding_left(24)
            .border_left(2, p.border)
            .color(p.foreground)
    })
}

/// `bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold`: code in prose.
///
/// The node stands on its own; inside a paragraph's text use a
/// `kivact::view::span` with the same family and background instead.
#[track_caller]
pub fn inline_code(value: impl IntoText) -> impl kivact::view::Component {
    node()
        .style(
            style()
                .align_self(Align::Start)
                .padding_x(4.8)
                .padding_y(3.2)
                .radius(0)
                .background(ColorName::Muted)
                .color(ColorName::Foreground)
                .font_family(MONO)
                .font_size(14)
                .weight(600)
                .text_wrap(false),
        )
        .child(text(value))
}

/// `ml-6 list-disc [&>li]:mt-2`: a bulleted list of `items` (`Role::List`).
#[track_caller]
pub fn bullet_list(items: impl IntoValue<Vec<String>>) -> impl kivact::view::Component {
    let items = items.into_value();
    column(for_index(items, |_, item| {
        row((
            text("•").style(style().width(16).shrink(0)),
            text(item).style(style().grow(1).min_width(0)),
        ))
        .with_node(|root| root.role(Role::ListItem))
        .name(item)
        .style(style().direction(Direction::Row).align_items(Align::Start))
    }))
    .with_node(|root| root.role(Role::List))
    .style(
        style()
            .gap(8)
            .padding_left(8)
            .font_size(16)
            .line_height(28.0 / 16.0)
            .color(ColorName::Foreground),
    )
}
