//! Role: The command palette recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::Shadow;
use kivact_core::style::{Align, FlexDirection, Position, Style, Theme, pct, style};
use kivact_interaction::{ListItemState, PopupState};
/// The layer that covers the window behind the modal.
pub(crate) fn scrim(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(kivact_core::style::color::OVERLAY)
}

/// The modal's panel: the search over the list.
pub(crate) fn panel(t: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(560)
        .max_width(pct(95))
        .background(kivact_core::style::color::RAISED)
        .color(kivact_core::style::color::TEXT)
        .border(1, kivact_core::style::color::BORDER)
        .border_radius(t.get(&kivact_core::style::radius::LG_VAR))
        .shadow(Shadow::new(0, 10, 30, kivact_core::style::color::SHADOW))
        .clip(true)
}

/// The row of the magnifier and the search field.
pub(crate) fn search(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .padding_x(t.get(&kivact_core::style::space::LG_VAR))
        .height(t.get(&kivact_core::style::control::HEIGHT_VAR) + 8.0)
        .border_bottom(1, kivact_core::style::color::BORDER)
}

/// The magnifier before the field.
pub(crate) fn search_icon(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .size(16, 16)
        .flex_shrink(0)
        .color(kivact_core::style::color::TEXT_MUTED)
}

/// The search field itself, without a box of its own.
pub(crate) fn field(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_grow(1)
        .min_width(0)
        .border_width(0)
        .background(kivact_core::style::Color::TRANSPARENT)
}

/// The scrolling list of rows.
pub(crate) fn list(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .max_height(320)
        .padding(t.get(&kivact_core::style::space::SM_VAR))
}

/// The heading of a group.
pub(crate) fn group(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding_x(t.get(&kivact_core::style::space::MD_VAR))
        .padding_y(t.get(&kivact_core::style::space::SM_VAR))
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

/// A command's row, marked while it is the active one.
pub(crate) fn row(t: &Theme, state: &Look<ListItemState>) -> Style {
    let s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .min_height(t.get(&kivact_core::style::control::HEIGHT_VAR))
        .padding_x(t.get(&kivact_core::style::space::MD_VAR))
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .color(kivact_core::style::color::TEXT)
        // A row is never a focus stop (the field keeps the focus), so a
        // disabled one takes the theme's opacity here.
        .disabled(style().opacity(t.get(&kivact_core::style::control::DISABLED_OPACITY_VAR)))
        .selected(style().background(kivact_core::style::color::SELECTION));
    if state.input.is_hovered() && !state.input.is_disabled() {
        s.background(kivact_core::style::color::HOVER)
    } else {
        s
    }
}

/// The icon at the start of a row.
pub(crate) fn icon(_: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .size(16, 16)
        .flex_shrink(0)
        .color(kivact_core::style::color::TEXT_MUTED)
}

/// The hint at the end of a row, such as its shortcut.
pub(crate) fn hint(t: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .flex_shrink(0)
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

/// The text shown when no command matches.
pub(crate) fn empty(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding(t.get(&kivact_core::style::space::XL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

/// The line under the list.
pub(crate) fn footer(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .padding_x(t.get(&kivact_core::style::space::LG_VAR))
        .padding_y(t.get(&kivact_core::style::space::MD_VAR))
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
        .border_top(1, kivact_core::style::color::BORDER)
}

recipe_table! {
    /// The recipes of a [`command_palette`](super::command_palette).
    CommandPaletteTheme {
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
