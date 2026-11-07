//! Role: SVG 1.1 export of a drawing, with the same `Draw` vocabulary as `Offscreen`.

mod api;
mod emit;
mod emitter;
mod encode;
mod fill;
mod image;
mod shape;
mod text;

#[doc(inline)]
pub use api::Svg;
