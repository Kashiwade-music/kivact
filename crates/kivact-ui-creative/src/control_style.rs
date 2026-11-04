//! Role: Creative field, check and segment style helpers.
//!
//! The focus ring and the disabled opacity are the theme's defaults
//! (`focus_ring`, `disabled_opacity`), which every part of an interaction
//! takes, a roving segment included.
use kivact::style::{Align, ColorToken, Direction, Style, TextOverflow, Theme, rgba, style};
pub(crate) fn check_style(t: &Theme) -> Style {
    let p = t.ext::<crate::tokens::Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(7)
        .height(p.row)
        .radius(5)
        .color(p.text)
}

pub(crate) fn field_style(t: &Theme) -> Style {
    let p = t.ext::<crate::tokens::Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(6)
        .height(p.row)
        .padding_x(9)
        .radius(7)
        .background(p.field)
        .color(p.text)
        .border(1, rgba(0xffffff, 0.04))
        .hover(style().background(ColorToken::Hover))
}

pub(crate) fn segmented_style(t: &Theme) -> Style {
    style()
        .direction(Direction::Row)
        .gap(2)
        .padding(2)
        .radius(9)
        .background(rgba(0x000000, 0.28))
        .border(1, t.ext::<crate::tokens::Tokens>().hair)
}

pub(crate) fn segmented_option_style(t: &Theme) -> Style {
    let p = t.ext::<crate::tokens::Tokens>();
    style()
        .direction(Direction::Row)
        .min_width(0)
        .text_wrap(false)
        .text_overflow(TextOverflow::Ellipsis)
        .gap(5)
        .height(p.row - 4.0)
        .padding_x(10)
        .radius(5)
        .align_items(Align::Center)
        .color(p.muted)
        .hover(style().background(ColorToken::Hover).color(p.text_high))
        .selected(style().background(p.strong).color(p.text_high))
}
