//! Role: Style navigation parts from the theme and passive metrics.
use crate::Look;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
pub(super) fn root<S>(t: &Theme, _: &Look<S>) -> Style {
    let p = &t.ext::<crate::UiTokens>().pagination;
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.text().size)
        .color(ColorToken::Text)
}
pub(super) fn part<S>(_: &Theme, _: &Look<S>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(4)
}
pub(super) fn current(t: &Theme, l: &Look<bool>) -> Style {
    style()
        .color(if **l {
            t.colors().text
        } else {
            t.colors().text_muted
        })
        .weight(if **l { 600 } else { 400 })
}
pub(super) fn panel(t: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    let p = &t.ext::<crate::UiTokens>().navigation_menu;
    style()
        .direction(Direction::Column)
        .padding(p.panel_padding)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
}
