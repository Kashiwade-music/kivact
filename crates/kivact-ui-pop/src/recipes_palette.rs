//! Role: Halo recipes for the standard command palette.
//!
//! A wide panel with a borderless search field over a ruled list, rows with an
//! optional icon, and the hint on the right.

use crate::Tokens;
use kivact::draw::Shadow;
use kivact::interaction::{ListItemState, PopupState};
use kivact::primitive::PartState;
use kivact::style::{Align, Color, ColorToken, Direction, Style, Theme, pct, style};
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
        .position(kivact::style::Position::Fixed)
        .inset(0)
        .background(kivact::style::rgba(0x0f172a, 0.45))
}

fn panel(t: &Theme, _: &Look<PopupState>) -> Style {
    let p = t.get::<Tokens>();
    style()
        .direction(Direction::Column)
        .width(560)
        .max_width(pct(95))
        .background(p.surface)
        .border(1, p.line)
        .radius(10)
        .color(p.text)
        .shadow(Shadow::new(0, 10, 15, ColorToken::Shadow))
        .clip(true)
}

fn search(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .border_bottom(1, ColorToken::Border)
}

/// Halo shows no magnifier.
fn hidden(_: &Theme, _: &Look<PartState>) -> Style {
    style().size(0, 0)
}

fn field(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .grow(1)
        .min_width(0)
        .height(48)
        .padding_x(12)
        .border(0, Color::TRANSPARENT)
        .radius(0)
}

fn list(_: &Theme, _: &Look<PartState>) -> Style {
    style().direction(Direction::Column).max_height(320)
}

fn group(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding_x(8)
        .padding_top(10)
        .padding_bottom(4)
        .font_size(11)
        .weight(600)
        .color(ColorToken::TextMuted)
}

fn row(t: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .height(36)
        .padding_x(8)
        .radius(6)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8)
        .selected(style().background(ColorToken::SurfaceAlt))
        // A row is never a focus stop (the field keeps the focus), so a
        // disabled one takes the theme's opacity here.
        .disabled(style().opacity(t.disabled_opacity()))
}

fn icon(_: &Theme, _: &Look<ListItemState>) -> Style {
    style().size(16, 16).shrink(0)
}

fn hint(_: &Theme, _: &Look<ListItemState>) -> Style {
    style().font_size(12).color(ColorToken::TextMuted)
}

fn empty(_: &Theme, _: &Look<PartState>) -> Style {
    style().padding(24).color(ColorToken::TextMuted)
}

fn footer(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding_x(12)
        .padding_y(8)
        .font_size(12)
        .color(ColorToken::TextMuted)
        .border_top(1, ColorToken::Border)
}
