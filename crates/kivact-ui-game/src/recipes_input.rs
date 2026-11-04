//! Role: Game recipes for the standard tag input and number input.

use crate::Tokens;
use kivact::custom::PartState;
use kivact::style::{Align, Color, ColorToken, Direction, Style, TextAlign, Theme, pct, style};
use kivact::ui::{Look, NumberInputRecipes, TagInputRecipes};

pub(crate) fn tag_input(r: TagInputRecipes) -> TagInputRecipes {
    r.root(|_| tag_root)
        .tag(|_| tag)
        .remove(|_| tag_remove)
        .input(|_| tag_field)
}

pub(crate) fn number_input(r: NumberInputRecipes) -> NumberInputRecipes {
    r.root(|_| number_root).input(|_| number_field)
}

fn tag_root(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Row)
        .width(pct(100))
        .wrap(true)
        .align_items(Align::Center)
        .gap(6)
        .padding(6)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
}

fn tag(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .direction(Direction::Row)
        .height(24)
        .padding_left(7)
        .gap(4)
        .align_items(Align::Center)
        .background(ColorToken::Selection)
        .border(1, ColorToken::Border)
        .font_size(11.5)
}

fn tag_remove(_: &Theme, _: &Look<PartState>) -> Style {
    style().size(20, 20)
}

fn tag_field(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .width(130)
        .min_width(90)
        .grow(1)
        .border_width(0)
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .ring(0, Color::TRANSPARENT),
        )
}

fn number_root(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .align_items(Align::Center)
        .border(1, t.ext::<Tokens>().line_strong)
        .background(t.colors().surface)
        .clip(true)
}

fn number_field(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .width(64)
        .border_width(0)
        .text_align(TextAlign::Center)
        .font_family("monospace")
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .ring(0, Color::TRANSPARENT),
        )
}
