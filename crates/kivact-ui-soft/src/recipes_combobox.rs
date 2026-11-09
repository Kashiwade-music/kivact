//! Role: Apply Neutral recipes to the standard keyed suggestion family.
use crate::Tokens;
use crate::recipes_field::{frame, text};
use kivact::custom::PartState;
use kivact::interaction::ListItemState;
use kivact::style::{Align, Color, Style, Theme, pct, style};
use kivact::ui::{ComboboxRecipes, Look};

pub(crate) fn combobox(r: ComboboxRecipes) -> ComboboxRecipes {
    r.frame(|_| field)
        .input(|_| input)
        .surface(|_| surface)
        .option(|_| option)
        .chip(|_| chip)
        .remove(|_| remove)
        .empty(|_| empty)
}
fn field(theme: &Theme, s: &Look<PartState>) -> Style {
    let t = theme.ext::<Tokens>();
    let base = frame(&t, s, false)
        .min_height(t.sizes.control_md)
        .padding_x(t.spacing.s2)
        .padding_y(t.spacing.s1 - t.sizes.border_width)
        .gap(t.spacing.s1)
        .align_items(Align::Center)
        .wrap(true);
    if s.is_focused() {
        base.border_color(t.colors.accent)
            .shadow(t.shadows.input_focus_ring)
    } else {
        base
    }
}
fn input(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .grow(1)
        .min_width(64)
        .height(24)
        .padding(0)
        .border(0, Color::TRANSPARENT)
        .ring_none()
        .background(Color::TRANSPARENT)
        .hover(style().ring_none())
        .focus(style().ring_none())
}
fn surface(theme: &Theme, _: &Look<PartState>) -> Style {
    let t = theme.ext::<Tokens>();
    t.shadows.low.iter().copied().fold(
        style()
            .width(pct(100))
            .max_height(300)
            .padding(t.spacing.s1)
            .radius(t.radius.container)
            .background(t.colors.bg_popover),
        Style::shadow,
    )
}
fn option(theme: &Theme, s: &Look<ListItemState>) -> Style {
    let t = theme.ext::<Tokens>();
    let base = text(
        style()
            .min_height(32)
            .padding_x(t.spacing.s2)
            .padding_y(t.spacing.s1_5)
            .radius(t.radius.element),
        t.typography.body,
        t.colors.text_primary,
    );
    if s.active {
        base.background(t.colors.overlay_hover)
    } else {
        base
    }
}
fn chip(theme: &Theme, _: &Look<PartState>) -> Style {
    let t = theme.ext::<Tokens>();
    style()
        .height(24)
        .padding_left(t.spacing.s2)
        .padding_right(t.spacing.s1)
        .gap(t.spacing.s1)
        .align_items(Align::Center)
        .radius(t.radius.element)
        .background(t.colors.accent_muted)
        .color(t.colors.text_primary)
}
fn remove(_: &Theme, _: &Look<PartState>) -> Style {
    style().size(16, 16).padding(0)
}
fn empty(theme: &Theme, _: &Look<PartState>) -> Style {
    let t = theme.ext::<Tokens>();
    style().padding(t.spacing.s2).color(t.colors.text_secondary)
}
