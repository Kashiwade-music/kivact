//! Role: Default recipes for the command palette.
use kivact_core::draw::Shadow;
use kivact_core::primitive::PartState;
use kivact_core::style::{Align, ColorToken, Direction, Position, Style, Theme, pct, style};
use kivact_interaction::{ListItemState, PopupState};

/// The layer that covers the window behind the modal.
pub(super) fn scrim(_: &Theme, _: &PartState) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(ColorToken::Overlay)
}

/// The modal's panel: the search over the list.
pub(super) fn panel(t: &Theme, _: &PopupState) -> Style {
    style()
        .direction(Direction::Column)
        .width(560)
        .max_width(pct(95))
        .background(ColorToken::Raised)
        .color(ColorToken::Text)
        .border(1, ColorToken::Border)
        .radius(t.radius().lg)
        .shadow(Shadow::new(0, 10, 30, ColorToken::Shadow))
        .clip(true)
}

/// The row of the magnifier and the search field.
pub(super) fn search(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .padding_x(t.space().lg)
        .height(t.control_height() + 8.0)
        .border_bottom(1, ColorToken::Border)
}

/// The magnifier before the field.
pub(super) fn search_icon(_: &Theme, _: &PartState) -> Style {
    style().size(16, 16).shrink(0).color(ColorToken::TextMuted)
}

/// The search field itself, without a box of its own.
pub(super) fn field(_: &Theme, _: &PartState) -> Style {
    style()
        .grow(1)
        .min_width(0)
        .border_width(0)
        .background(kivact_core::style::Color::TRANSPARENT)
}

/// The scrolling list of rows.
pub(super) fn list(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .max_height(320)
        .padding(t.space().sm)
}

/// The heading of a group.
pub(super) fn group(t: &Theme, _: &PartState) -> Style {
    style()
        .padding_x(t.space().md)
        .padding_y(t.space().sm)
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// A command's row, marked while it is the active one.
pub(super) fn row(t: &Theme, state: &ListItemState) -> Style {
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .min_height(t.control_height())
        .padding_x(t.space().md)
        .radius(t.radius().sm)
        .color(ColorToken::Text)
        // A row is never a focus stop (the field keeps the focus), so a
        // disabled one takes the theme's opacity here.
        .disabled(style().opacity(t.disabled_opacity()))
        .selected(style().background(ColorToken::Selection));
    if state.input.hovered && !state.input.disabled {
        s.background(ColorToken::Hover)
    } else {
        s
    }
}

/// The icon at the start of a row.
pub(super) fn icon(_: &Theme, _: &ListItemState) -> Style {
    style().size(16, 16).shrink(0).color(ColorToken::TextMuted)
}

/// The hint at the end of a row, such as its shortcut.
pub(super) fn hint(t: &Theme, _: &ListItemState) -> Style {
    style()
        .shrink(0)
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// The text shown when no command matches.
pub(super) fn empty(t: &Theme, _: &PartState) -> Style {
    style().padding(t.space().xl).color(ColorToken::TextMuted)
}

/// The line under the list.
pub(super) fn footer(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .padding_x(t.space().lg)
        .padding_y(t.space().md)
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
        .border_top(1, ColorToken::Border)
}
