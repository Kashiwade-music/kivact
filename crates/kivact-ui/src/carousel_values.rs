//! Role: Define passive carousel presentation modes and recipe projections.
use kivact_core::view::{Orientation, ReadingDirection};

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

/// The structural layout used by a carousel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselMode {
    #[default]
    Translate,
    Scroll,
}
