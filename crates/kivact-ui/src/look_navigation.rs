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

pub(super) fn stepper_layout(_: &Theme, _: &Look<crate::StepperItemState>) -> Style {
    style().direction(Direction::Row)
}
pub(super) fn stepper_body(_: &Theme, _: &Look<crate::StepperItemState>) -> Style {
    style().direction(Direction::Column)
}
pub(super) fn breadcrumb_list(_: &Theme, _: &Look<()>) -> Style {
    style().direction(Direction::Row).gap(8)
}
pub(super) fn breadcrumb_item(t: &Theme, s: &Look<crate::BreadcrumbItemState>) -> Style {
    style()
        .padding(0)
        .height(kivact_core::style::auto())
        .min_width(0)
        .border(0, kivact_core::style::Color::TRANSPARENT)
        .background(kivact_core::style::Color::TRANSPARENT)
        .color(if s.is_current() {
            t.colors().text
        } else {
            t.colors().text_muted
        })
        .weight(400)
}

pub(super) fn pagination_item(t: &Theme, s: &Look<crate::PaginationItemState>) -> Style {
    style()
        .color(if s.current {
            t.colors().text
        } else {
            t.colors().text_muted
        })
        .weight(if s.current { 600 } else { 400 })
}

pub(super) fn stepper_decoration(_: &Theme, _: &Look<crate::StepperItemState>) -> Style {
    style().size(0, 0)
}

pub(super) fn pagination_wrapper(_: &Theme, _: &Look<crate::PaginationSummary>) -> Style {
    style().display(kivact_core::style::Display::Contents)
}
