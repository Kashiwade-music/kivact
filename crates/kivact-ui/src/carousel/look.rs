//! Role: The carousel recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{FlexDirection, Style, Theme, style};
use kivact_core::style::{Length, ThemeVar};
use kivact_core::view::{Orientation, ReadingDirection};

/// The gap between the slides of a carousel, in logical px.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::new("carousel.gap", 16.0);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);
/// The padding of a carousel's viewport, in logical px.
pub static PADDING_VAR: ThemeVar<f64> = ThemeVar::new("carousel.padding", 0.0);
/// [`PADDING_VAR`] as a length for a style.
pub const PADDING: Length = Length::var(&PADDING_VAR);
/// The width of the fade at a carousel's edges, in logical px.
pub static EDGE_FADE_WIDTH_VAR: ThemeVar<f64> = ThemeVar::new("carousel.edge_fade_width", 4.0);
/// [`EDGE_FADE_WIDTH_VAR`] as a length for a style.
pub const EDGE_FADE_WIDTH: Length = Length::var(&EDGE_FADE_WIDTH_VAR);

pub(crate) fn carousel_gap(t: &Theme) -> f64 {
    t.get(&GAP_VAR)
}

pub(crate) fn carousel_padding(t: &Theme) -> f64 {
    t.get(&PADDING_VAR)
}

pub(crate) fn carousel_fade_width(t: &Theme) -> f64 {
    t.get(&EDGE_FADE_WIDTH_VAR)
}

pub(crate) fn carousel_root(_: &Theme, _: &Look<CarouselSurfaceState>) -> Style {
    style().flex_direction(FlexDirection::Column).min_width(0)
}

pub(crate) fn carousel_track(t: &Theme, _: &Look<CarouselSurfaceState>) -> Style {
    style().gap(carousel_gap(t)).transition(
        kivact_core::style::TransitionProp::All,
        kivact_core::motion::Transition::tween(300, kivact_core::motion::Easing::EaseOut),
    )
}

pub(crate) fn carousel_viewport(t: &Theme, _: &Look<CarouselSurfaceState>) -> Style {
    style().padding(carousel_padding(t))
}

pub(crate) fn carousel_slide(_: &Theme, _: &Look<CarouselSurfaceState>) -> Style {
    style()
}

pub(crate) fn carousel_control(_: &Theme, _: &Look<CarouselSurfaceState>) -> Style {
    style()
}

/// Passive navigation and layout facts supplied to carousel recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CarouselSurfaceState {
    pub state: kivact_interaction::CarouselState,
    pub orientation: Orientation,
    pub direction: ReadingDirection,
}

impl std::ops::Deref for CarouselSurfaceState {
    type Target = kivact_interaction::CarouselState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

recipe_table! {
    /// Styles for a carousel's persistent parts.
    CarouselTheme {
        root(root_recipe): CarouselSurfaceState = carousel_root,
        viewport(viewport_recipe): CarouselSurfaceState = carousel_viewport,
        track(track_recipe): CarouselSurfaceState = carousel_track,
        slide(slide_recipe): CarouselSurfaceState = carousel_slide,
        previous(previous_recipe): CarouselSurfaceState = carousel_control,
        next(next_recipe): CarouselSurfaceState = carousel_control,
    }
}
