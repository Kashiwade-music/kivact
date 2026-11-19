//! Role: The command palette recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::draw::Shadow;
use kivact_core::style::{Align, ColorToken, Direction, Position, Style, Theme, pct, style};
use kivact_interaction::{ListItemState, PopupState};
/// The layer that covers the window behind the modal.
pub(crate) fn scrim(_: &Theme, _: &PartState) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(ColorToken::Overlay)
}

/// The modal's panel: the search over the list.
pub(crate) fn panel(t: &Theme, _: &PopupState) -> Style {
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
pub(crate) fn search(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .padding_x(t.space().lg)
        .height(t.control_height() + 8.0)
        .border_bottom(1, ColorToken::Border)
}

/// The magnifier before the field.
pub(crate) fn search_icon(_: &Theme, _: &PartState) -> Style {
    style().size(16, 16).shrink(0).color(ColorToken::TextMuted)
}

/// The search field itself, without a box of its own.
pub(crate) fn field(_: &Theme, _: &PartState) -> Style {
    style()
        .grow(1)
        .min_width(0)
        .border_width(0)
        .background(kivact_core::style::Color::TRANSPARENT)
}

/// The scrolling list of rows.
pub(crate) fn list(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .max_height(320)
        .padding(t.space().sm)
}

/// The heading of a group.
pub(crate) fn group(t: &Theme, _: &PartState) -> Style {
    style()
        .padding_x(t.space().md)
        .padding_y(t.space().sm)
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// A command's row, marked while it is the active one.
pub(crate) fn row(t: &Theme, state: &ListItemState) -> Style {
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
    if state.input.is_hovered() && !state.input.is_disabled() {
        s.background(ColorToken::Hover)
    } else {
        s
    }
}

/// The icon at the start of a row.
pub(crate) fn icon(_: &Theme, _: &ListItemState) -> Style {
    style().size(16, 16).shrink(0).color(ColorToken::TextMuted)
}

/// The hint at the end of a row, such as its shortcut.
pub(crate) fn hint(t: &Theme, _: &ListItemState) -> Style {
    style()
        .shrink(0)
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// The text shown when no command matches.
pub(crate) fn empty(t: &Theme, _: &PartState) -> Style {
    style().padding(t.space().xl).color(ColorToken::TextMuted)
}

/// The line under the list.
pub(crate) fn footer(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .padding_x(t.space().lg)
        .padding_y(t.space().md)
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
        .border_top(1, ColorToken::Border)
}

recipe_table! {
    /// The recipes of a [`command_palette`](super::command_palette).
    CommandPaletteRecipes {
        /// The layer that covers the window behind the modal.
        scrim(scrim_recipe): PartState = scrim,
        /// The modal's panel, which holds the search, the list and the footer.
        panel(panel_recipe): PopupState = panel,
        /// The row of the magnifier and the search field.
        search(search_recipe): PartState = search,
        /// The magnifier before the field, drawn while the theme's `IconSet`
        /// has a `search` image. A look without one draws none, and a look
        /// that does not want it gives the box no size.
        search_icon(search_icon_recipe): PartState = search_icon,
        /// The search field (a native editing leaf: its style has no paint).
        field(field_recipe): PartState = field,
        /// The scrolling box of the groups and rows.
        list(list_recipe): PartState = list,
        /// The heading of a group.
        group(group_recipe): PartState = group,
        /// A command's row; `active` marks the row Enter runs, and the row
        /// is `selected` while it is.
        row(row_recipe): ListItemState = row,
        /// The icon at the start of a row.
        icon(icon_recipe): ListItemState = icon,
        /// The hint at the end of a row.
        hint(hint_recipe): ListItemState = hint,
        /// The text shown when no command matches.
        empty(empty_recipe): PartState = empty,
        /// The line under the list, drawn only when the caller gives a
        /// [`footer`](super::CommandPalette::footer).
        footer(footer_recipe): PartState = footer,
    }
}
