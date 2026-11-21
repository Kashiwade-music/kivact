//! Role: Lyra recipes of the text input and text area, and of a field's label, hint and error.
//!
//! The look of input.tsx, textarea.tsx and the field's text beside them.

use crate::color::{DESTRUCTIVE, FOREGROUND, INPUT, MUTED_FOREGROUND, PRIMARY};
use crate::recipe_parts::{field_fill, focus_ring, invalid_edge};
use crate::variants::InputGroupControl;
use kivact::style::{FlexDirection, Style, Theme, ThemeMode, color, pct, style};
use kivact::ui::ControlSize;
use kivact::ui::theme::{
    FieldState, FieldTheme, Look, TextAreaTheme, TextInputState, TextInputTheme,
};

/// The field look of input.tsx and textarea.tsx.
fn field(t: &Theme, state: &Look<TextInputState>) -> Style {
    let dark = t.mode() == ThemeMode::Dark;
    let s = style()
        .min_width(0.0)
        .padding_x(10.0)
        .border_radius(0.0)
        .border(1.0, INPUT)
        .background(field_fill(dark))
        .color(FOREGROUND)
        .font_size(12.0)
        .placeholder_color(MUTED_FOREGROUND)
        .caret_color(FOREGROUND)
        .selection_color(PRIMARY.with_alpha(0.2))
        .disabled(style().background(INPUT.with_alpha(if dark { 0.8 } else { 0.5 })));
    if state.invalid {
        let (border, ring) = invalid_edge(dark);
        s.border_color(border).outline(1.0, ring)
    } else {
        // A field shows its focus however it got it.
        s.focus(focus_ring(t))
    }
}

/// The single-line input: 24 to 36 px high by size.
fn text_input(t: &Theme, state: &Look<TextInputState>, _base: Style) -> Style {
    let base = field(t, state)
        .height(match state.size {
            ControlSize::ExtraSmall => 24.0,
            ControlSize::Small => 28.0,
            ControlSize::Medium => 32.0,
            ControlSize::Large => 36.0,
        })
        .padding_y(4.0);
    if state.variant::<InputGroupControl>().is_some() {
        crate::recipes_input::control(base, false)
    } else {
        base
    }
}

/// `min-h-16 field-sizing-content`: at least 64 px, growing with the lines
/// of the value (a native text area does not wrap, and it is never lower
/// than three lines).
fn text_area(t: &Theme, state: &Look<TextInputState>, _base: Style) -> Style {
    let base = field(t, state)
        .min_height(64.0)
        .padding_y(8.0)
        .line_height(16.0 / 12.0);
    if state.variant::<InputGroupControl>().is_some() {
        crate::recipes_input::control(base, true)
    } else {
        base
    }
}

/// The column of a field: its label, control and hint.
fn field_root(_: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100.0))
        .min_width(0.0)
        .gap(8.0)
}

/// A field's label: `text-xs font-medium`, in destructive while invalid.
fn field_label(_: &Theme, state: &Look<FieldState>, _base: Style) -> Style {
    style()
        .font_size(12.0)
        .font_weight(500)
        .line_height(1.0)
        .color(if state.invalid {
            DESTRUCTIVE
        } else {
            color::TEXT
        })
}

/// A field's hint: `text-xs`, muted.
fn field_hint(_: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style().font_size(12.0).color(color::TEXT_MUTED)
}

/// A field's error: `text-xs`, in destructive.
fn field_error(_: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style().font_size(12.0).color(DESTRUCTIVE)
}

/// Installs the text input, the text area and the field parts.
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
}
