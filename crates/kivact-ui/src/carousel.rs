//! Role: Render binding-driven translated or native scrolling carousels.

mod geometry;
pub(crate) mod look;
mod view;

pub use view::{Carousel, CarouselMode, carousel};
