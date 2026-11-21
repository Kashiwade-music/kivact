//! Role: Share sizing and look helpers that two or more components recipes use.

use crate::control_size::ControlSize;
use crate::variants::Look;
use kivact_core::custom::PartState;
use kivact_core::style::Shadow;
use kivact_core::style::{Align, Color, FlexDirection, Style, Theme, style};
use kivact_interaction::PopupState;
/// The font size of a control.
pub(crate) fn font_size(theme: &Theme, size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall | ControlSize::Small => {
            theme.get(&kivact_core::style::font::SIZE_SMALL_VAR)
        }
        ControlSize::Medium => theme.get(&kivact_core::style::font::SIZE_VAR),
        ControlSize::Large => theme.get(&kivact_core::style::font::SIZE_VAR) + 2.0,
    }
}

/// The height of a control: the theme's `control_height`, 80% of it for
/// a small one and 65% for an extra small one.
pub(crate) fn height(theme: &Theme, size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => {
            (theme.get(&kivact_core::style::control::HEIGHT_VAR) * 0.65).round()
        }
        ControlSize::Small => (theme.get(&kivact_core::style::control::HEIGHT_VAR) * 0.8).round(),
        ControlSize::Medium => theme.get(&kivact_core::style::control::HEIGHT_VAR),
        ControlSize::Large => theme.get(&kivact_core::style::control::HEIGHT_VAR) + 8.0,
    }
}

/// The gap between a control's parts.
pub(crate) fn gap(theme: &Theme, size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => theme.get(&kivact_core::style::space::XS_VAR),
        ControlSize::Small => theme.get(&kivact_core::style::space::SM_VAR),
        ControlSize::Medium | ControlSize::Large => theme.get(&kivact_core::style::space::MD_VAR),
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
                .outline(0, Color::TRANSPARENT),
        )
}

pub(crate) fn trigger(t: &Theme) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .height(t.get(&kivact_core::style::control::HEIGHT_VAR))
        .padding_x(t.get(&kivact_core::style::space::MD_VAR))
        .border(1, kivact_core::style::color::BORDER)
        .border_radius(t.get(&kivact_core::style::radius::MD_VAR))
        .background(kivact_core::style::color::SURFACE)
}

pub(crate) fn surface(t: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .padding(4)
        .background(kivact_core::style::color::SURFACE)
        .border(1, kivact_core::style::color::BORDER)
        .border_radius(t.get(&kivact_core::style::radius::MD_VAR))
        .min_width(160)
        .max_height(320)
}

/// The secondary line of an option or a menu item.
pub(crate) fn description(t: &Theme) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

/// The box of a mark icon, centred.
pub(crate) fn mark() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(kivact_core::style::Justify::Center)
        .flex_shrink(0)
}

/// The items in a column.
pub(crate) fn list(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(t.get(&kivact_core::style::space::XS_VAR))
}

pub(crate) fn part<S>(_: &Theme, _: &Look<S>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(4)
}

pub(crate) fn side(size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => 20.0,
        ControlSize::Small => 24.0,
        ControlSize::Medium => 36.0,
        ControlSize::Large => 48.0,
    }
}

pub(crate) fn tone(tone: kivact_interaction::Tone) -> Color {
    match tone {
        kivact_interaction::Tone::Neutral => kivact_core::style::color::TEXT_MUTED,
        kivact_interaction::Tone::Info | kivact_interaction::Tone::Accent => {
            kivact_core::style::color::ACCENT
        }
        kivact_interaction::Tone::Success => kivact_core::style::color::SUCCESS,
        kivact_interaction::Tone::Warning => kivact_core::style::color::WARNING,
        kivact_interaction::Tone::Danger => kivact_core::style::color::DANGER,
    }
}

pub(crate) fn column<S>(t: &Theme, _: &Look<S>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .min_width(0)
}

pub(crate) fn row<S>(t: &Theme, _: &Look<S>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .align_items(Align::Center)
        .min_width(0)
}

pub(crate) fn slot<S>(_: &Theme, _: &Look<S>) -> Style {
    style().min_width(0)
}

pub(crate) fn muted<S>(t: &Theme, _: &Look<S>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR) - 2.0)
        .color(kivact_core::style::color::TEXT_MUTED)
}
