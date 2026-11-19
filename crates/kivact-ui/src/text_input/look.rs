//! Role: The text input recipe state, tokens, recipe table and default looks.
use crate::look_common::{font_size, height};

use crate::control_size::ControlSize;
use kivact_core::custom::PartState;
use kivact_core::style::{Color, ColorToken, Style, Theme, style};
/// The text input recipe: a bordered field that rings on focus.
pub(crate) fn text_input(theme: &Theme, state: &TextInputState) -> Style {
    let size = state.size;
    let s = style()
        .height(height(theme, size))
        .min_height(0)
        .padding_x(theme.space().md)
        .radius(theme.radius().md)
        .border(1, ColorToken::Border)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .font_size(font_size(theme, size))
        // A field shows its focus however it got it, as a browser's does.
        .focus(style().border_color(ColorToken::Focus).ring_with_offset(
            theme.focus_ring(),
            theme.focus_ring_offset(),
            Color::token(ColorToken::Focus).with_alpha(0.35),
        ));
    if state.invalid {
        s.border_color(ColorToken::Danger)
    } else {
        s
    }
}

pub(crate) fn text_area(theme: &Theme, state: &TextInputState) -> Style {
    text_input(theme, state)
        .height(100)
        .padding_y(theme.space().sm)
}

/// The state of a text input or a text area for its recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextInputState {
    pub input: PartState,
    pub size: ControlSize,
    /// Whether the caller marked the value invalid (`TextInput::invalid`).
    pub invalid: bool,
}
