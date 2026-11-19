//! Role: The carousel recipe state, tokens, recipe table and default looks.

use crate::UiTokens;
use crate::recipe::recipe_table;
use kivact_core::style::{Direction, Style, Theme, style};
use kivact_core::view::{Orientation, ReadingDirection};
pub(crate) fn carousel_gap(t: &Theme) -> f64 {
    t.ext::<UiTokens>().carousel.gap.unwrap_or(16.0)
}

pub(crate) fn carousel_padding(t: &Theme) -> f64 {
    t.ext::<UiTokens>().carousel.padding.unwrap_or(0.0)
}

pub(crate) fn carousel_fade_width(t: &Theme) -> f64 {
    t.ext::<UiTokens>().carousel.edge_fade_width.unwrap_or(4.0)
}

pub(crate) fn carousel_root(_: &Theme, _: &CarouselSurfaceState) -> Style {
    style().direction(Direction::Column).min_width(0)
}

pub(crate) fn carousel_track(t: &Theme, _: &CarouselSurfaceState) -> Style {
    style().gap(carousel_gap(t)).transition(
        kivact_core::style::TransitionProp::All,
        kivact_core::motion::Transition::tween(300, kivact_core::motion::Easing::EaseOut),
    )
}

pub(crate) fn carousel_viewport(t: &Theme, _: &CarouselSurfaceState) -> Style {
    style().padding(carousel_padding(t))
}

pub(crate) fn carousel_slide(_: &Theme, _: &CarouselSurfaceState) -> Style {
    style()
}

pub(crate) fn carousel_control(_: &Theme, _: &CarouselSurfaceState) -> Style {
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

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CarouselTokens {
    /// Space between slides; None uses the standard recipe.
    pub gap: Option<f64>,
    /// Viewport inset; None uses the standard recipe.
    pub padding: Option<f64>,
    /// Fade distance at overflowing edges.
    pub edge_fade_width: Option<f64>,
}

recipe_table! {
    /// Styles for a carousel's persistent parts.
    CarouselRecipes {
        root(root_recipe): CarouselSurfaceState = carousel_root,
        viewport(viewport_recipe): CarouselSurfaceState = carousel_viewport,
        track(track_recipe): CarouselSurfaceState = carousel_track,
        slide(slide_recipe): CarouselSurfaceState = carousel_slide,
        previous(previous_recipe): CarouselSurfaceState = carousel_control,
        next(next_recipe): CarouselSurfaceState = carousel_control,
    }
}
