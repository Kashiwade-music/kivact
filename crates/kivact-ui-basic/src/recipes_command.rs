//! Role: Lyra recipes for the standard command palette (command.tsx).

use crate::Color;
use crate::layers::scrim_style;
use crate::theme_access::{list_row, menu_shortcut, popover_surface};
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, PopupState};
use kivact::style::{Align, Direction, Overflow, Position, Style, TextAlign, Theme, pct, style};
use kivact::ui::{CommandPaletteRecipes, Look};

pub(crate) fn command_palette(r: CommandPaletteRecipes) -> CommandPaletteRecipes {
    r.scrim(|_| scrim)
        .panel(|_| panel)
        .search(|_| search)
        .search_icon(|_| search_icon)
        .field(|_| field)
        .list(|_| list)
        .group(|_| group)
        .row(|_| row)
        .icon(|_| icon)
        .hint(|_| hint)
        .empty(|_| empty)
        .footer(|_| footer)
}

fn scrim(_: &Theme, _: &Look<PartState>) -> Style {
    scrim_style()
}

/// The palette: `bg-popover` in the popover surface, 384 px wide, a third of
/// the way down the window.
fn panel(t: &Theme, _: &Look<PopupState>) -> Style {
    popover_surface(&t.ext::<Tokens>())
        .position(Position::Fixed)
        .top(pct(100.0 / 3.0))
        .left(pct(50))
        .translate(pct(-50), 0)
        .direction(Direction::Column)
        .width(384)
        .max_width(pct(100) - 32)
        .overflow(Overflow::Hidden)
}

/// `h-8` over a `border-b`, on `bg-input/30`.
fn search(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .shrink(0)
        .height(32)
        .padding_left(8)
        .border_bottom(1, Color::Border)
        .background(t.ext::<Tokens>().input.with_alpha(0.3))
}

fn search_icon(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .size(16, 16)
        .shrink(0)
        .opacity(0.5)
        .color(Color::Foreground)
}

fn field(t: &Theme, _: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .grow(1)
        .min_width(0)
        .height(pct(100))
        .padding_left(6)
        .padding_right(8)
        .font_size(12)
        .color(p.popover_foreground)
        .placeholder_color(p.muted_foreground)
        .caret_color(p.foreground)
        .selection_color(p.primary.with_alpha(0.2))
}

fn list(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Column)
        .max_height(288)
        .min_height(0)
}

fn group(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding_x(8)
        .padding_y(6)
        .font_size(12)
        .color(Color::MutedForeground)
}

/// A row: `px-2 py-2 text-xs gap-2`, `bg-muted` while active.
fn row(t: &Theme, s: &Look<ListItemState>) -> Style {
    let p = t.ext::<Tokens>();
    let base = list_row(&p, false);
    if s.active {
        base.background(p.muted).color(p.foreground)
    } else {
        base
    }
}

fn icon(_: &Theme, _: &Look<ListItemState>) -> Style {
    style().size(16, 16).shrink(0)
}

fn hint(t: &Theme, s: &Look<ListItemState>) -> Style {
    let p = t.ext::<Tokens>();
    let base = menu_shortcut(&p, false).shrink(0);
    if s.active {
        base.color(p.foreground)
    } else {
        base
    }
}

fn empty(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding_y(24)
        .align_self(Align::Stretch)
        .text_align(TextAlign::Center)
        .font_size(12)
        .color(Color::PopoverForeground)
}

fn footer(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding(8)
        .font_size(12)
        .color(Color::MutedForeground)
        .border_top(1, Color::Border)
}
