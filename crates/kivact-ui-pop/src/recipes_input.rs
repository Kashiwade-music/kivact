//! Role: Pop recipes for the standard tag input and number input.

use crate::{Hue, Tokens};
use kivact::custom::PartState;
use kivact::draw::Shadow;
use kivact::style::{Align, Color, Direction, Style, TextAlign, Theme, style};
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
        .min_height(36)
        .padding_x(6)
        .padding_y(4)
        .gap(4)
        .wrap(true)
        .align_items(Align::Center)
        .background(crate::Color::Surface)
        .border(1, crate::Color::Line)
        .radius(8)
}

fn tag(t: &Theme, _: &Look<PartState>) -> Style {
    let (fill, text) = t.ext::<Tokens>().hue(Hue::Blue);
    style()
        .direction(Direction::Row)
        .height(26)
        .padding_left(10)
        .padding_right(4)
        .gap(2)
        .align_items(Align::Center)
        .radius(6)
        .background(fill)
        .color(text)
        .font_size(13)
}

fn tag_remove(_: &Theme, _: &Look<PartState>) -> Style {
    style().size(18, 18).radius(4)
}

fn tag_field(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .height(26)
        .padding_y(4)
        .padding_x(4)
        .min_width(100)
        .width(130)
        .grow(1)
        .border(0, Color::TRANSPARENT)
        .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .ring(0, Color::TRANSPARENT),
        )
}

fn number_root(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .height(36)
        .align_items(Align::Center)
        .border(1, crate::Color::Line)
        .radius(8)
        .background(crate::Color::Surface)
        .clip(true)
}

fn number_field(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .grow(1)
        .height(34)
        .min_width(40)
        .width(40)
        .border(0, Color::TRANSPARENT)
        .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
        .text_align(TextAlign::Center)
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .ring(0, Color::TRANSPARENT),
        )
}
