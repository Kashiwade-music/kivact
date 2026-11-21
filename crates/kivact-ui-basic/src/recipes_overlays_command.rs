//! Role: Lyra recipes of the command palette (command.tsx).

use crate::color::{
    BORDER, FOREGROUND, INPUT, MUTED, MUTED_FOREGROUND, POPOVER_FOREGROUND, PRIMARY,
};
use crate::recipe_parts::{list_row, popover_surface, scrim_style};
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, PopupState};
use kivact::style::{
    Align, FlexDirection, Overflow, Position, Style, TextAlign, Theme, pct, style,
};
use kivact::ui::theme::{CommandPaletteTheme, Look};

fn scrim(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    scrim_style()
}

/// The palette: `bg-popover` in the popover surface, 384 px wide, a third of
/// the way down the window.
fn panel(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    popover_surface()
        .position(Position::Fixed)
        .top(pct(100.0 / 3.0))
        .left(pct(50.0))
        .translate(pct(-50.0), 0.0)
        .flex_direction(FlexDirection::Column)
        .width(384.0)
        .max_width(pct(100.0) - 32.0)
        .overflow(Overflow::Hidden)
}

/// `h-8` over a `border-b`, on `bg-input/30`.
fn search(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .flex_shrink(0.0)
        .height(32.0)
        .padding_left(8.0)
        .border_bottom(1.0, BORDER)
        .background(INPUT.with_alpha(0.3))
}

fn search_icon(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .size(16.0, 16.0)
        .flex_shrink(0.0)
        .opacity(0.5)
        .color(FOREGROUND)
}

fn field(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_grow(1.0)
        .min_width(0.0)
        .height(pct(100.0))
        .padding_left(6.0)
        .padding_right(8.0)
        .font_size(12.0)
        .color(POPOVER_FOREGROUND)
        .placeholder_color(MUTED_FOREGROUND)
        .caret_color(FOREGROUND)
        .selection_color(PRIMARY.with_alpha(0.2))
}

fn list(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .max_height(288.0)
        .min_height(0.0)
}

fn group(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .padding_x(8.0)
        .padding_y(6.0)
        .font_size(12.0)
        .color(MUTED_FOREGROUND)
}

/// A row: `px-2 py-2 text-xs gap-2`, `bg-muted` while active.
fn row(_: &Theme, s: &Look<ListItemState>, _base: Style) -> Style {
    let base = list_row(false);
    if s.active {
        base.background(MUTED).color(FOREGROUND)
    } else {
        base
    }
}

fn icon(_: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style().size(16.0, 16.0).flex_shrink(0.0)
}

fn hint(_: &Theme, s: &Look<ListItemState>, _base: Style) -> Style {
    let base = crate::recipe_parts::menu_shortcut(false).flex_shrink(0.0);
    if s.active {
        base.color(FOREGROUND)
    } else {
        base
    }
}

fn empty(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .padding_y(24.0)
        .align_self(Align::Stretch)
        .text_align(TextAlign::Center)
        .font_size(12.0)
        .color(POPOVER_FOREGROUND)
}

fn footer(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .padding(8.0)
        .font_size(12.0)
        .color(MUTED_FOREGROUND)
        .border_top(1.0, BORDER)
}

/// The command palette recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CommandPaletteTheme::default()
            .scrim(scrim)
            .panel(panel)
            .search(search)
            .search_icon(search_icon)
            .field(field)
            .list(list)
            .group(group)
            .row(row)
            .icon(icon)
            .hint(hint)
            .empty(empty)
            .footer(footer),
    )
}
