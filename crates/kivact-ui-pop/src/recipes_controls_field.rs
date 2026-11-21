//! Role: The Pop looks of the text fields, the number input and the field labels.

use crate::color;
use kivact::custom::PartState;
use kivact::style::Shadow;
use kivact::style::color as core;
use kivact::style::{Align, Color, FlexDirection, Style, TextAlign, Theme, pct, style};
use kivact::ui::ControlSize;
use kivact::ui::theme::{
    FieldState, FieldTheme, Look, NumberInputTheme, TextAreaTheme, TextInputState, TextInputTheme,
};

/// Install the kit's recipes of the text fields, the number input and the field.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(TextInputTheme::default().root(text_input))
        .with(TextAreaTheme::default().root(text_area))
        .with(
            NumberInputTheme::default()
                .root(number_root)
                .input(number_field),
        );
    theme.with(
        FieldTheme::default()
            .root(field_root)
            .label(field_label)
            .hint(field_hint)
            .error(field_error),
    )
}

/// The surface of a text input: a field of the kit's look, with a blue border while focused.
fn input_style() -> Style {
    style()
        .height(36)
        .min_height(0)
        .padding_x(12)
        .padding_y(8)
        .border_radius(8)
        .border(1, color::LINE)
        .background(color::SURFACE)
        .color(color::TEXT)
        .font_size(14)
        .line_height(1.2)
        .shadow(Shadow::new(0, 1, 2, color::SUBTLE_SHADOW))
        .focus(style().border_color(color::BLUE).outline(3, core::FOCUS))
}

fn text_input(_theme: &Theme, state: &Look<TextInputState>, _base: Style) -> Style {
    let s = input_style();
    match state.size {
        ControlSize::ExtraSmall => s.height(26),
        ControlSize::Small => s.height(32),
        ControlSize::Medium | ControlSize::Large => s,
    }
}

fn text_area(theme: &Theme, state: &Look<TextInputState>, base: Style) -> Style {
    text_input(theme, state, base)
        .height(80)
        .padding_y(8)
        .line_height(1.5)
}

fn number_root(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .height(36)
        .align_items(Align::Center)
        .border(1, color::LINE)
        .border_radius(8)
        .background(color::SURFACE)
        .clip(true)
}

fn number_field(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_grow(1)
        .height(34)
        .min_width(40)
        .width(40)
        .border(0, Color::TRANSPARENT)
        .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
        .text_align(TextAlign::Center)
        .focus(
            style()
                .border_color(Color::TRANSPARENT)
                .outline(0, Color::TRANSPARENT),
        )
}

fn field_root(_theme: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .min_width(0)
        .gap(8)
}

fn field_label(_theme: &Theme, state: &Look<FieldState>, _base: Style) -> Style {
    style()
        .font_size(14)
        .font_weight(500)
        .line_height(1.)
        .color(if state.invalid {
            core::DANGER
        } else {
            core::TEXT
        })
}

fn field_hint(_theme: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style().font_size(13).color(core::TEXT_MUTED)
}

fn field_error(_theme: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style().font_size(13).color(core::DANGER)
}
