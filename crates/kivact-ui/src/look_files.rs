//! Role: Supply default carousel and drop target recipes from theme values.
use crate::UiTokens;
use kivact_core::style::{Align, BorderStyle, ColorToken, Direction, Style, Theme, style};
use kivact_interaction::DropzoneState;
pub(super) fn carousel_gap(t: &Theme) -> f64 {
    t.ext::<UiTokens>().carousel.gap.unwrap_or(16.0)
}
pub(super) fn carousel_padding(t: &Theme) -> f64 {
    t.ext::<UiTokens>().carousel.padding.unwrap_or(0.0)
}
pub(super) fn carousel_fade_width(t: &Theme) -> f64 {
    t.ext::<UiTokens>().carousel.edge_fade_width.unwrap_or(4.0)
}
pub(super) fn carousel_root(_: &Theme, _: &crate::carousel_values::CarouselSurfaceState) -> Style {
    style().direction(Direction::Column).min_width(0)
}
pub(super) fn carousel_track(t: &Theme, _: &crate::carousel_values::CarouselSurfaceState) -> Style {
    style().gap(carousel_gap(t)).transition(
        kivact_core::style::TransitionProp::All,
        kivact_core::motion::Transition::tween(300, kivact_core::motion::Easing::EaseOut),
    )
}
pub(super) fn carousel_viewport(
    t: &Theme,
    _: &crate::carousel_values::CarouselSurfaceState,
) -> Style {
    style().padding(carousel_padding(t))
}
pub(super) fn carousel_slide(_: &Theme, _: &crate::carousel_values::CarouselSurfaceState) -> Style {
    style()
}
pub(super) fn carousel_control(
    _: &Theme,
    _: &crate::carousel_values::CarouselSurfaceState,
) -> Style {
    style()
}
pub(super) fn dropzone_root(t: &Theme, s: &DropzoneState) -> Style {
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
pub(super) fn dropzone_label(_: &Theme, _: &DropzoneState) -> Style {
    style().weight(600)
}
pub(super) fn dropzone_hint(_: &Theme, _: &DropzoneState) -> Style {
    style().font_size(12).color(ColorToken::TextMuted)
}
pub(super) fn dropzone_status(_: &Theme, _: &DropzoneState) -> Style {
    style().font_size(12).color(ColorToken::Danger)
}

pub(super) fn dropzone_icon(_: &Theme, _: &DropzoneState) -> Style {
    style()
}
