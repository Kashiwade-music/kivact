//! Role: Style Astryx Neutral text fields, number inputs and field text (the controls group's input parts).
use crate::color as c;
use crate::metrics as m;
use crate::recipe_parts::{editor, flat, frame, text};
use crate::space as s;
use crate::type_scale as t;
use kivact::custom::PartState;
use kivact::style::{Align, FlexDirection, Style, Theme, pct, style};
use kivact::ui::ControlSize;
use kivact::ui::theme::{
    FieldState, FieldTheme, Look, NumberInputTheme, TextAreaTheme, TextInputState, TextInputTheme,
};

/// The height of a text field at `size`.
fn height(size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall | ControlSize::Small => m::CONTROL_SM,
        ControlSize::Medium => m::CONTROL_MD,
        ControlSize::Large => m::CONTROL_LG,
    }
}

fn text_input(theme: &Theme, look: &Look<TextInputState>, _base: Style) -> Style {
    editor(frame(theme, &look.input, look.invalid))
        .height(height(look.size))
        .padding_x(s::SPACE_S2)
        .padding_y(s::SPACE_S1)
}

fn text_area(theme: &Theme, look: &Look<TextInputState>, _base: Style) -> Style {
    const DEFAULT_ROWS: f64 = 3.0;
    let block = if look.size == ControlSize::Large {
        theme.get(&s::SPACE_S2_VAR)
    } else {
        theme.get(&s::SPACE_S1_VAR)
    };
    editor(frame(theme, &look.input, look.invalid))
        .width(pct(100))
        .min_height(DEFAULT_ROWS * t::BODY.line_height_px + 2.0 * (block + m::BORDER_WIDTH))
        .padding_x(s::SPACE_S2)
        .padding_y(block)
}

fn field_root(_theme: &Theme, _look: &Look<FieldState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .min_width(0)
        .gap(s::SPACE_S1)
}

fn field_label(_theme: &Theme, look: &Look<FieldState>, _base: Style) -> Style {
    let color = if look.invalid {
        c::TEXT_ERROR
    } else {
        c::TEXT_SECONDARY
    };
    text(style(), t::LABEL, color)
}

fn field_hint(_theme: &Theme, _look: &Look<FieldState>, _base: Style) -> Style {
    text(style(), t::SUPPORTING, c::TEXT_SECONDARY)
}

fn field_error(_theme: &Theme, _look: &Look<FieldState>, _base: Style) -> Style {
    text(style(), t::SUPPORTING, c::TEXT_ERROR)
}

/// The box of the number input: its frame, with the input between two side buttons.
fn number_root(theme: &Theme, look: &Look<PartState>, _base: Style) -> Style {
    // The standard spinbutton retains its two side buttons.
    frame(theme, look, false)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .height(m::CONTROL_MD)
        .clip(true)
}

fn number_input(theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    flat(theme)
        .height(m::CONTROL_MD - 2.0 * m::BORDER_WIDTH)
        .padding_x(s::SPACE_S2)
        .flex_grow(1)
}

/// Install the kit's recipes of the field, the text fields and the number input.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            FieldTheme::default()
                .root(field_root)
                .label(field_label)
                .hint(field_hint)
                .error(field_error),
        )
        .with(TextInputTheme::default().root(text_input))
        .with(TextAreaTheme::default().root(text_area))
        .with(
            NumberInputTheme::default()
                .root(number_root)
                .input(number_input),
        )
}
