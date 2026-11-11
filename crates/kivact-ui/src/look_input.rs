//! Role: Default recipes for the tag input and the number input.
use kivact_core::custom::PartState;
use kivact_core::draw::Shadow;
use kivact_core::style::{Align, Color, ColorToken, Direction, Style, TextAlign, Theme, style};

/// The box of the tags and the input, ringed while the focus is inside.
pub(super) fn tag_root(t: &Theme, state: &PartState) -> Style {
    let s = style()
        .direction(Direction::Row)
        .wrap(true)
        .align_items(Align::Center)
        .gap(t.space().sm)
        .min_height(t.control_height())
        .padding(t.space().sm)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
        .color(ColorToken::Text);
    if state.is_focused() {
        s.border_color(ColorToken::Focus).ring_with_offset(
            t.focus_ring(),
            t.focus_ring_offset(),
            Color::token(ColorToken::Focus).with_alpha(0.35),
        )
    } else {
        s
    }
}

/// One tag: its text and its remove button.
pub(super) fn tag(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().xs)
        .padding_left(t.space().md)
        .padding_right(t.space().xs)
        .min_height(t.control_height() - 10.0)
        .radius(t.radius().sm)
        .background(ColorToken::Selection)
        .font_size(t.text().size_small)
}

/// A tag's remove button, small enough to sit in the tag.
pub(super) fn tag_remove(t: &Theme, _: &PartState) -> Style {
    style().size(18, 18).padding(0).radius(t.radius().sm)
}

/// The input among the tags, without a box of its own: the root's is the
/// field's.
pub(super) fn tag_input(t: &Theme, _: &PartState) -> Style {
    flat(t)
        .grow(1)
        .min_width(96)
        .height(t.control_height() - 10.0)
}

/// The decrease button, the input and the increase button in one box.
pub(super) fn number_root(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .height(t.control_height())
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .clip(true)
}

/// The number, centered, without a box of its own.
pub(super) fn number_input(t: &Theme, _: &PartState) -> Style {
    flat(t)
        .grow(1)
        .min_width(40)
        .width(56)
        .height(t.control_height() - 2.0)
        .text_align(TextAlign::Center)
}

/// A text input that draws neither a border nor a ring of its own.
fn flat(_: &Theme) -> Style {
    style()
        .border(0, Color::TRANSPARENT)
        .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
        .background(Color::TRANSPARENT)
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .ring(0, Color::TRANSPARENT),
        )
}
