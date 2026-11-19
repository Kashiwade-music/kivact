//! Role: The dropzone recipe state, tokens, recipe table and default looks.

use crate::UiTokens;
use crate::recipe::recipe_table;
use kivact_core::style::{Align, BorderStyle, ColorToken, Direction, Style, Theme, style};
use kivact_interaction::DropzoneState;
pub(crate) fn dropzone_root(t: &Theme, s: &DropzoneState) -> Style {
    let tokens = t.ext::<UiTokens>();
    let base = style()
        .direction(Direction::Column)
        .align_items(Align::Center)
        .gap(6)
        .padding(tokens.dropzone.padding.unwrap_or(16.0))
        .min_height(tokens.dropzone.min_height.unwrap_or(96.0))
        .border(1, t.colors().border)
        .border_style(BorderStyle::Dashed);
    match s {
        DropzoneState::DragOver {
            valid: kivact_interaction::Validity::Invalid(_),
        } => base.border_color(ColorToken::Danger),
        DropzoneState::DragOver { .. } => base.border_color(ColorToken::Accent),
        DropzoneState::Failed(_) => base.border_color(ColorToken::Danger),
        _ => base,
    }
}

pub(crate) fn dropzone_label(_: &Theme, _: &DropzoneState) -> Style {
    style().weight(600)
}

pub(crate) fn dropzone_hint(_: &Theme, _: &DropzoneState) -> Style {
    style().font_size(12).color(ColorToken::TextMuted)
}

pub(crate) fn dropzone_status(_: &Theme, _: &DropzoneState) -> Style {
    style().font_size(12).color(ColorToken::Danger)
}

pub(crate) fn dropzone_icon(_: &Theme, _: &DropzoneState) -> Style {
    style()
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DropzoneTokens {
    /// Root padding; None uses the standard recipe.
    pub padding: Option<f64>,
    /// Minimum target height; None uses the standard recipe.
    pub min_height: Option<f64>,
}

recipe_table! {
    /// Styles for a file drop target and its feedback.
    DropzoneRecipes {
        root(root_recipe): DropzoneState = dropzone_root,
        icon(icon_recipe): DropzoneState = dropzone_icon,
        label(label_recipe): DropzoneState = dropzone_label,
        hint(hint_recipe): DropzoneState = dropzone_hint,
        status(status_recipe): DropzoneState = dropzone_status,
    }
}
