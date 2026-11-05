//! Role: Expose carousel state and navigation without drawing slides.
mod api;
mod machine;
pub use api::{CarouselApi, Snap, use_carousel};
pub use machine::CarouselState;
