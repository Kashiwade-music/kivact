//! Role: Render binding-driven translated or native scrolling carousels.

mod geometry;
pub(crate) mod look;
pub(crate) mod tokens;
mod view;

pub use kivact_interaction::Snap;
pub use view::{Carousel, CarouselMode, carousel};
