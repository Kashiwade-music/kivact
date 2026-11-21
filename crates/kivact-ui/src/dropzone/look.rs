//! Role: The dropzone recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{Align, BorderStyle, FlexDirection, Style, Theme, style};
use kivact_core::style::{Length, ThemeVar};
use kivact_interaction::DropzoneState;

/// The padding of a dropzone, in logical px.
pub static PADDING_VAR: ThemeVar<f64> = ThemeVar::new("dropzone.padding", 16.0);
/// [`PADDING_VAR`] as a length for a style.
pub const PADDING: Length = Length::var(&PADDING_VAR);

/// The minimum height of a dropzone, in logical px.
pub static MIN_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("dropzone.min_height", 96.0);
/// [`MIN_HEIGHT_VAR`] as a length for a style.
pub const MIN_HEIGHT: Length = Length::var(&MIN_HEIGHT_VAR);

pub(crate) fn dropzone_root(t: &Theme, s: &Look<DropzoneState>) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .gap(6)
        .padding(PADDING)
        .min_height(MIN_HEIGHT)
        .border(1, t.get(&kivact_core::style::color::BORDER_VAR))
        .border_style(BorderStyle::Dashed);
    match &**s {
        DropzoneState::DragOver {
            valid: kivact_interaction::Validity::Invalid(_),
        } => base.border_color(kivact_core::style::color::DANGER),
        DropzoneState::DragOver { .. } => base.border_color(kivact_core::style::color::ACCENT),
        DropzoneState::Failed(_) => base.border_color(kivact_core::style::color::DANGER),
        _ => base,
    }
}

pub(crate) fn dropzone_label(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style().font_weight(600)
}

pub(crate) fn dropzone_hint(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style()
        .font_size(12)
        .color(kivact_core::style::color::TEXT_MUTED)
}

pub(crate) fn dropzone_status(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style()
        .font_size(12)
        .color(kivact_core::style::color::DANGER)
}

pub(crate) fn dropzone_icon(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style()
}

recipe_table! {
    /// Styles for a file drop target and its feedback.
    DropzoneTheme {
        root(root_recipe): DropzoneState = dropzone_root,
        icon(icon_recipe): DropzoneState = dropzone_icon,
        label(label_recipe): DropzoneState = dropzone_label,
        hint(hint_recipe): DropzoneState = dropzone_hint,
        status(status_recipe): DropzoneState = dropzone_status,
    }
}
