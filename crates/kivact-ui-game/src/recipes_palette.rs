//! Role: Game recipes for the standard command palette.
//!
//! A square panel with a lit search field, rows with an accent bar on the
//! active one, and the hint in mono type on the right.

use crate::Tokens;
use kivact::draw::Shadow;
use kivact::interaction::{ListItemState, PopupState};
use kivact::primitive::PartState;
use kivact::style::{Align, ColorToken, Direction, Position, Style, Theme, pct, style};
use kivact::ui::{CommandPaletteRecipes, Look};

pub(crate) fn command_palette(r: CommandPaletteRecipes) -> CommandPaletteRecipes {
    r.scrim(|_| scrim)
        .panel(|_| panel)
        .search(|_| search)
        .search_icon(|_| hidden)
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
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(ColorToken::Overlay)
        .backdrop_blur(2)
}

fn panel(t: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .direction(Direction::Column)
        .width(600)
        .max_width(pct(100) - 32)
        .background(t.colors().surface)
        .border(1, t.get::<Tokens>().line_strong)
        .shadow(Shadow::new(0, 14, 36, t.colors().shadow))
        .clip(true)
}

fn search(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .border_bottom(2, ColorToken::Accent)
}

/// The console shows no magnifier.
fn hidden(_: &Theme, _: &Look<PartState>) -> Style {
    style().size(0, 0)
}

fn field(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .grow(1)
        .min_width(0)
        .height(52)
        .font_size(15)
        .border_width(0)
}

fn list(_: &Theme, _: &Look<PartState>) -> Style {
    style().direction(Direction::Column).max_height(360)
}

fn group(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding_x(10)
        .padding_top(10)
        .padding_bottom(4)
        .font_family(t.get::<Tokens>().mono_family.as_str())
        .font_size(10.5)
        .weight(600)
        .color(ColorToken::TextMuted)
}

fn row(t: &Theme, _: &Look<ListItemState>) -> Style {
    // A row is never a focus stop (the field keeps the focus), so a
    // disabled one takes the theme's opacity here.
    style()
        .direction(Direction::Row)
        .height(38)
        .padding_x(10)
        .gap(10)
        .align_items(Align::Center)
        .selected(
            style()
                .background(ColorToken::Selection)
                .border_left(3, ColorToken::Accent)
                .color(ColorToken::Accent),
        )
        .disabled(style().opacity(t.disabled_opacity()))
}

fn icon(_: &Theme, _: &Look<ListItemState>) -> Style {
    style().size(16, 16).shrink(0)
}

fn hint(t: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .font_family(t.get::<Tokens>().mono_family.as_str())
        .font_size(10.5)
        .weight(600)
        .color(ColorToken::TextMuted)
}

fn empty(_: &Theme, _: &Look<PartState>) -> Style {
    style().padding(28).color(ColorToken::TextMuted)
}

fn footer(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding_x(14)
        .padding_y(8)
        .font_family(t.get::<Tokens>().mono_family.as_str())
        .font_size(10.5)
        .weight(600)
        .background(ColorToken::SurfaceAlt)
        .border_top(1, ColorToken::Border)
}
