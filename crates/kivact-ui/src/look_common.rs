//! Role: Share sizing and look helpers that two or more components recipes use.

use crate::Look;
use crate::control_size::ControlSize;
use kivact_core::custom::PartState;
use kivact_core::draw::Shadow;
use kivact_core::style::{Align, Color, ColorToken, Direction, Style, Theme, style};
use kivact_interaction::PopupState;
/// The font size of a control.
pub(crate) fn font_size(theme: &Theme, size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall | ControlSize::Small => theme.text().size_small,
        ControlSize::Medium => theme.text().size,
        ControlSize::Large => theme.text().size + 2.0,
    }
}

/// The height of a control: the theme's `control_height`, 80% of it for
/// a small one and 65% for an extra small one.
pub(crate) fn height(theme: &Theme, size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => (theme.control_height() * 0.65).round(),
        ControlSize::Small => (theme.control_height() * 0.8).round(),
        ControlSize::Medium => theme.control_height(),
        ControlSize::Large => theme.control_height() + 8.0,
    }
}

/// The gap between a control's parts.
pub(crate) fn gap(theme: &Theme, size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => theme.space().xs,
        ControlSize::Small => theme.space().sm,
        ControlSize::Medium | ControlSize::Large => theme.space().md,
    }
}

/// A text input that draws neither a border nor a ring of its own.
pub(crate) fn flat(_: &Theme) -> Style {
    style()
        .border(0, Color::TRANSPARENT)
        .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
        .background(Color::TRANSPARENT)
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .ring(0, Color::TRANSPARENT),
        )
}

pub(crate) fn trigger(t: &Theme) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .height(t.control_height())
        .padding_x(t.space().md)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
}

pub(crate) fn surface(t: &Theme, _: &PopupState) -> Style {
    style()
        .padding(4)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .min_width(160)
        .max_height(320)
}

/// The secondary line of an option or a menu item.
pub(crate) fn description(t: &Theme) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// The box of a mark icon, centred.
pub(crate) fn mark() -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(kivact_core::style::Justify::Center)
        .shrink(0)
}

/// The items in a column.
pub(crate) fn list(t: &Theme, _: &PartState) -> Style {
    style().direction(Direction::Column).gap(t.space().xs)
}

pub(crate) fn part<S>(_: &Theme, _: &Look<S>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(4)
}
