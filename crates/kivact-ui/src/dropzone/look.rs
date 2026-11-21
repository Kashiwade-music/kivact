//! Role: The dropzone recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::tokens::UiTokens;
use kivact_core::style::{Align, BorderStyle, FlexDirection, Style, Theme, style};
use kivact_interaction::DropzoneState;
pub(crate) fn dropzone_root(t: &Theme, s: &DropzoneState) -> Style {
    let tokens = t.ext::<UiTokens>();
    let base = style()
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .gap(6)
        .padding(tokens.dropzone.padding.unwrap_or(16.0))
        .min_height(tokens.dropzone.min_height.unwrap_or(96.0))
        .border(1, t.get(&kivact_core::style::color::BORDER_VAR))
        .border_style(BorderStyle::Dashed);
    match s {
        DropzoneState::DragOver {
            valid: kivact_interaction::Validity::Invalid(_),
        } => base.border_color(kivact_core::style::color::DANGER),
        DropzoneState::DragOver { .. } => base.border_color(kivact_core::style::color::ACCENT),
        DropzoneState::Failed(_) => base.border_color(kivact_core::style::color::DANGER),
        _ => base,
    }
}

pub(crate) fn dropzone_label(_: &Theme, _: &DropzoneState) -> Style {
    style().font_weight(600)
}

pub(crate) fn dropzone_hint(_: &Theme, _: &DropzoneState) -> Style {
    style()
        .font_size(12)
        .color(kivact_core::style::color::TEXT_MUTED)
}

pub(crate) fn dropzone_status(_: &Theme, _: &DropzoneState) -> Style {
    style()
        .font_size(12)
        .color(kivact_core::style::color::DANGER)
}

pub(crate) fn dropzone_icon(_: &Theme, _: &DropzoneState) -> Style {
    style()
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
