//! Role: The field recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{FlexDirection, Style, Theme, style};
/// A field: its label, control, hint and error stacked.
pub(crate) fn field(t: &Theme, _: &Look<FieldState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(t.get(&kivact_core::style::space::XS_VAR))
}

/// A field's visible label, in the danger color while the field has an error.
pub(crate) fn field_label(t: &Theme, state: &Look<FieldState>) -> Style {
    let s = style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .font_weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR));
    if state.invalid {
        s.color(kivact_core::style::color::DANGER)
    } else {
        s
    }
}

/// A field's hint: muted small text under the control.
pub(crate) fn field_hint(t: &Theme, _: &Look<FieldState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

/// A field's error message: small text in the danger color.
pub(crate) fn field_error(t: &Theme, _: &Look<FieldState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::DANGER)
}

/// The state of a [`field`](super::field) for the recipes of its parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldState {
    /// Whether the field shows an error message (`Field::error` is `Some`).
    pub invalid: bool,
}

recipe_table! {
    /// The recipes of a [`field`](super::field).
    FieldTheme {
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
