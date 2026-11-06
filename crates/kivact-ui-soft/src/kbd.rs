//! Role: Style the standard shortcut row and its individual Neutral key caps.
use crate::Tokens;
use kivact::style::{Align, Justify, Style, Theme, style};
use kivact::ui::{KbdKeyState, Look};
pub(crate) fn root(_: &Theme, _: &Look<()>) -> Style {
    style().align_items(Align::Center).gap(4).shrink(0)
}
pub(crate) fn key(theme: &Theme, _: &Look<KbdKeyState>) -> Style {
    let t = theme.ext::<Tokens>();
    style()
        .height(20)
        .min_width(20)
        .padding_x(t.spacing.s1)
        .radius(t.radius.inner)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(t.colors.neutral)
        .border_bottom(2, t.colors.border_emphasized)
        .color(t.colors.text_primary)
        .font_size(12)
        .line_height(20.0 / 12.0)
        .weight(t.typography.weight_medium)
        .text_wrap(false)
}
