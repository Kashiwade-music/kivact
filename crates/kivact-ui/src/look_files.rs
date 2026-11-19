//! Role: Supply default carousel and drop target recipes from theme values.
use crate::UiTokens;
use kivact_core::style::{Direction, Style, Theme, style};
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
