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
pub(super) fn breadcrumb_root(t: &Theme, _: &Look<()>) -> Style {
    let p = &t.ext::<crate::UiTokens>().breadcrumb;
    style()
        .direction(Direction::Row)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.text().size)
        .color(ColorToken::Text)
}
pub(super) fn stepper_root(t: &Theme, _: &Look<crate::navigation_values::StepperState>) -> Style {
    let p = &t.ext::<crate::UiTokens>().stepper;
    style()
        .direction(Direction::Row)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.text().size)
        .color(ColorToken::Text)
}
pub(super) fn navigation_root(t: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    let p = &t.ext::<crate::UiTokens>().navigation_menu;
    style()
        .direction(Direction::Row)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.text().size)
        .color(ColorToken::Text)
}
pub(super) fn stepper_item(
    t: &Theme,
    look: &Look<crate::navigation_values::StepperItemState>,
) -> Style {
    let p = &t.ext::<crate::UiTokens>().stepper;
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_width(if look.collapsed {
            0.0
        } else {
            p.min_step_width
        })
        .gap(p.connector_gap)
        .color(if look.index == look.current {
            t.colors().accent
        } else {
            t.colors().text
        })
        .opacity(if look.disabled {
            t.disabled_opacity()
        } else {
            1.0
        })
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

pub(super) fn stepper_layout(_: &Theme, _: &Look<crate::StepperItemState>) -> Style {
    style().direction(Direction::Row)
}
pub(super) fn stepper_body(_: &Theme, _: &Look<crate::StepperItemState>) -> Style {
    style().direction(Direction::Column)
}
