//! Role: shadcn/ui typography styles for headings, prose, quotes, code and lists.
//!
//! The styles of the shadcn/ui typography page. They are not part of the
//! Lyra preset's 12 px controls: prose uses 16 px body text. Vertical
//! spacing between blocks is the caller's (`column(..).style(style().gap(..))`).

use crate::color::{BORDER, FOREGROUND, MUTED, MUTED_FOREGROUND};
use crate::recipe_parts::MONO;

use kivact::component;
use kivact::state::Value;
use kivact::style::{Align, FlexDirection, FontStyle, style};
use kivact::view::{Element, column, node, row, text};

use kivact::view::{Role, for_index};

/// `leading-7`: a 16 px paragraph.
#[component]
pub fn paragraph(value: Value<String>) -> impl Element {
    text(value).style(
        style()
            .font_size(16)
            .line_height(28.0 / 16.0)
            .color(FOREGROUND),
    )
}

/// `text-xl text-muted-foreground`: an introductory paragraph.
#[component]
pub fn lead(value: Value<String>) -> impl Element {
    text(value).style(
        style()
            .font_size(20)
            .line_height(28.0 / 20.0)
            .color(MUTED_FOREGROUND),
    )
}

/// `text-lg font-semibold`: emphasized text.
#[component]
pub fn large(value: Value<String>) -> impl Element {
    text(value).style(
        style()
            .font_size(18)
            .line_height(28.0 / 18.0)
            .font_weight(600)
            .color(FOREGROUND),
    )
}

/// `text-sm leading-none font-medium`: small print.
#[component]
pub fn small(value: Value<String>) -> impl Element {
    text(value).style(
        style()
            .font_size(14)
            .line_height(1.0)
            .font_weight(500)
            .color(FOREGROUND),
    )
}

/// `text-sm text-muted-foreground`: secondary text.
#[component]
pub fn muted(value: Value<String>) -> impl Element {
    text(value).style(
        style()
            .font_size(14)
            .line_height(20.0 / 14.0)
            .color(MUTED_FOREGROUND),
    )
}

/// `border-l-2 pl-6 italic`: a quotation.
#[component]
pub fn blockquote(value: Value<String>) -> impl Element {
    text(value).style(
        style()
            .font_size(16)
            .line_height(28.0 / 16.0)
            .font_style(FontStyle::Italic)
            .padding_left(24)
            .border_left(2, BORDER)
            .color(FOREGROUND),
    )
}

/// `bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold`: code in prose.
///
/// The node stands on its own; inside a paragraph's text use a
/// `kivact::view::span` with the same family and background instead.
#[component]
pub fn inline_code(value: Value<String>) -> impl Element {
    node()
        .style(
            style()
                .align_self(Align::Start)
                .padding_x(4.8)
                .padding_y(3.2)
                .border_radius(0.0)
                .background(MUTED)
                .color(FOREGROUND)
                .font_family(MONO)
                .font_size(14)
                .font_weight(600)
                .text_wrap(false),
        )
        .child(text(value))
}

/// `ml-6 list-disc [&>li]:mt-2`: a bulleted list of `items` (`Role::List`).
#[component]
pub fn bullet_list(items: Value<Vec<String>>) -> impl Element {
    column(for_index(items, |_, item| {
        row((
            text("•").style(style().width(16).flex_shrink(0.0)),
            text(item).style(style().flex_grow(1.0).min_width(0)),
        ))
        .with_node(|root| root.role(Role::ListItem))
        .name(item)
        .style(
            style()
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Start),
        )
    }))
    .with_node(|root| root.role(Role::List))
    .style(
        style()
            .gap(8)
            .padding_left(8)
            .font_size(16)
            .line_height(28.0 / 16.0)
            .color(FOREGROUND),
    )
}
