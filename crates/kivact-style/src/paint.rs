//! Role: Fills: colors, gradients and images, and the engine's fill layers.
//!
//! What boxes and drawings are filled with: colors, gradients and images,
//! sized, placed and repeated like CSS background layers. The same
//! [`Paint`] fills a box (`style().background(..)`, `mask(..)`) and, in
//! custom drawing, a shape (`Draw`); both resolve it here into the engine's
//! fill layer.
//!
//! The element paint vocabulary is CSS's property names with typed values:
//! `background` ([`Background`]: a stack of [`Paint`]s written as a tuple,
//! the first on top), `border`, `border_top` .. `border_left`,
//! `border_width`, `border_color` and `border_style` (solid, dashed,
//! dotted: width and color per side, one style for all sides), `radius`,
//! `radius_corners`, `radius_top_left` .. (`CornerRadius`: px, percent or
//! an `(x, y)` ellipse), `shadow` (`Shadow::new(x, y, blur,
//! color).spread(s).inset()`, the first on top), `opacity` (a group),
//! `filter`, `backdrop_filter` and `backdrop_blur` (`Filter`: a list of CSS
//! filter functions), `mix_blend_mode` (`BlendMode`), `mask` (a `Paint`)
//! and `clip_path` (`Shape`: polygon, inset, circle, ellipse). These values
//! are `Draw`'s too and live in `kivact::draw`, except `Background` and
//! `BorderStyle`, which only a style takes (`kivact::style`); the prelude
//! keeps the everyday names only.
//!
//! A `Paint` is one CSS background layer: a color, a [`Gradient`] or an
//! image, with its tile size, position and repeat; `Draw::fill_rect` fills
//! with the same type (percentages are of the filled rectangle). A gradient
//! is `linear_gradient(angle)` (CSS angles: 0 is up, 90 right),
//! `radial_gradient()` (circle or ellipse, extent, radius, center) or
//! `conic_gradient(start)`, optionally `repeating()`, with `stop(color,
//! position)` (px or percent, degrees for a conic one) or `colors([..])`
//! without positions. A calc `Length` (`pct(100) - 20`) is resolved when
//! drawn wherever a length is drawn (background size and position, gradient
//! centers and stops, corner radii, clip paths, masks).

mod background;
pub(super) mod canvas_gradient;
mod fill_layer;
mod gradient;
mod layer;

pub use background::Background;
pub use canvas_gradient::{CanvasGradient, GradientExtend};
pub(crate) use fill_layer::fill_layer;
pub use gradient::{Gradient, RadialExtent, conic_gradient, linear_gradient, radial_gradient};
pub use layer::Paint;
