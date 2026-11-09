//! Role: Lyra field, suggestion and chip recipes for the standard combobox.
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::style::{Align, Color, Direction, Justify, Style, Theme, pct, style};
use kivact::ui::{ComboboxRecipes, Look};

pub(crate) fn combobox(parts: ComboboxRecipes) -> ComboboxRecipes {
    parts
        .frame(|_| frame)
        .input(|_| input)
        .surface(|_| surface)
        .option(|_| option)
        .chip(|_| chip)
        .remove(|_| remove)
        .empty(|_| empty)
}
fn frame(t: &Theme, s: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    let base = style()
        .direction(Direction::Row)
        .wrap(true)
        .align_items(Align::Center)
        .gap(4)
        .width(pct(100))
        .min_height(32)
        .padding(4)
        .radius(0)
        .background(crate::theme_access::field_fill(&p))
        .border(1, p.input);
    let base = if s.disabled {
        base.merge(&crate::theme_access::disabled())
            .background(p.input.with_alpha(if p.is_dark() { 0.8 } else { 0.5 }))
    } else {
        base
    };
    if s.is_focused() {
        base.merge(&crate::theme_access::focus_ring(&p))
    } else {
        base
    }
}
fn input(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .grow(1)
        .min_width(64)
        .height(22)
        .padding_x(2)
        .padding_y(0)
        .border(0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .ring_none()
        .hover(style().ring_none())
        .focus(style().ring_none())
}
fn surface(t: &Theme, _: &Look<PartState>) -> Style {
    crate::list_popup::surface_style(&t.ext::<Tokens>()).max_height(288)
}
fn option(t: &Theme, s: &Look<kivact::interaction::ListItemState>) -> Style {
    crate::list_popup::row_style(&t.ext::<Tokens>(), s)
}
fn chip(t: &Theme, _: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(4)
        .height(21)
        .padding_left(6)
        .padding_right(2)
        .radius(0)
        .background(p.muted)
        .color(p.foreground)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
}
fn remove(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .size(16, 16)
        .padding(0)
        .opacity(0.5)
        .hover(style().opacity(1))
}
fn empty(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .width(pct(100))
        .padding_y(8)
        .font_size(12)
        .text_align(kivact::style::TextAlign::Center)
        .color(t.ext::<Tokens>().muted_foreground)
}
