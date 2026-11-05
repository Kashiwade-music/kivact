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
pub(super) fn stepper_root(t: &Theme, _: &Look<crate::StepperState>) -> Style {
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
pub(super) fn stepper_item(t: &Theme, look: &Look<crate::StepperItemState>) -> Style {
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
        .color(if s.current {
            t.colors().text
        } else {
            t.colors().text_muted
        })
        .weight(400)
}

pub(super) fn action(_: &Theme, _: &Look<kivact_interaction::NavOpen>) -> Style {
    style()
        .background(kivact_core::style::Color::TRANSPARENT)
        .color(ColorToken::Text)
        .border(0, kivact_core::style::Color::TRANSPARENT)
}
pub(super) fn link(t: &Theme, s: &Look<crate::navigation_values::NavigationLinkState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(if s.has_description {
            Align::Start
        } else {
            Align::Center
        })
        .gap(t.space().sm)
        .padding(t.space().sm)
        .color(ColorToken::Text)
        .hover(style().background(ColorToken::Hover))
        .focus_visible(style().ring(t.focus_ring(), ColorToken::Focus))
}
pub(super) fn link_title(
    _: &Theme,
    _: &Look<crate::navigation_values::NavigationLinkState>,
) -> Style {
    style().weight(600)
}
pub(super) fn link_description(
    t: &Theme,
    _: &Look<crate::navigation_values::NavigationLinkState>,
) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}
pub(super) fn link_icon(
    _: &Theme,
    _: &Look<crate::navigation_values::NavigationLinkState>,
) -> Style {
    style().shrink(0)
}

pub(super) fn stepper_layout(_: &Theme, _: &Look<crate::StepperItemState>) -> Style {
    style().direction(Direction::Row).min_width(0)
}
pub(super) fn stepper_decoration(_: &Theme, _: &Look<crate::StepperItemState>) -> Style {
    style().size(0, 0)
}
