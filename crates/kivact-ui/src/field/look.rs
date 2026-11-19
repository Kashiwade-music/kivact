//! Role: The field recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::style::{ColorToken, Direction, Style, Theme, style};
/// A field: its label, control, hint and error stacked.
pub(crate) fn field(t: &Theme, _: &FieldState) -> Style {
    style().direction(Direction::Column).gap(t.space().xs)
}

/// A field's visible label, in the danger color while the field has an error.
pub(crate) fn field_label(t: &Theme, state: &FieldState) -> Style {
    let s = style()
        .font_size(t.text().size_small)
        .weight(t.text().weight_strong);
    if state.invalid {
        s.color(ColorToken::Danger)
    } else {
        s
    }
}

/// A field's hint: muted small text under the control.
pub(crate) fn field_hint(t: &Theme, _: &FieldState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// A field's error message: small text in the danger color.
pub(crate) fn field_error(t: &Theme, _: &FieldState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::Danger)
}

/// The state of a [`field`](super::field) for the recipes of its parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldState {
    /// Whether the field shows an error message (`Field::error` is `Some`).
    pub invalid: bool,
}

recipe_table! {
    /// The recipes of a [`field`](super::field).
    FieldRecipes {
        /// The column of label, control, hint and error.
        root(root_recipe): FieldState = field,
        /// The visible label, which knows whether the field has an error.
        label(label_recipe): FieldState = field_label,
        /// The hint under the control.
        hint(hint_recipe): FieldState = field_hint,
        /// The error message under the hint.
        error(error_recipe): FieldState = field_error,
    }
}
