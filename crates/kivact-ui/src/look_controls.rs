//! Role: Default recipes for the radio group, slider, toasts and splitter.
use crate::control_size::ControlSize;
use crate::toggle::look::CheckableState;
use kivact_core::style::{Align, ColorToken, Direction, Justify, Style, Theme, style};
use kivact_core::view::Orientation;

/// The options in a column, or a row when horizontal.
pub(super) fn radio_group(t: &Theme, orientation: &Orientation) -> Style {
    let s = style().gap(t.space().md);
    match orientation {
        Orientation::Vertical => s.direction(Direction::Column),
        _ => s.direction(Direction::Row).wrap(true),
    }
}

/// An option: its indicator beside its label.
pub(super) fn radio_item(t: &Theme, state: &CheckableState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .radius(t.radius().sm)
        .font_size(match state.size {
            ControlSize::ExtraSmall | ControlSize::Small => t.text().size_small,
            ControlSize::Medium | ControlSize::Large => t.text().size,
        })
        .color(ColorToken::Text)
}

/// The circle of an option, the accent around the `dot` while chosen.
pub(super) fn radio_indicator(t: &Theme, state: &CheckableState) -> Style {
    let side = (t.text().size + 2.0).round();
    let s = style()
        .size(side, side)
        .shrink(0)
        .radius(side / 2.0)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(ColorToken::Surface);
    if state.is_checked() {
        s.border(1, ColorToken::Accent).color(ColorToken::Accent)
    } else if state.invalid {
        s.border(1, ColorToken::Danger)
    } else {
        s.border(1, ColorToken::Border)
    }
}

pub(super) fn radio_mark(_: &Theme, state: &CheckableState) -> Style {
    if state.is_checked() {
        style()
    } else {
        style().size(0, 0).opacity(0.0)
    }
}

pub(super) fn radio_description(t: &Theme, _: &CheckableState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}
