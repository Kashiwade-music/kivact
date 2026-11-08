//! Role: Fixtures shared by the end-to-end tests: mounting, colors, random.
//! Mounting helpers, the CSS color formulas of the color tests and the
//! deterministic random driver of the model tests.
//!
//! Pixel tiers. Each comparison of pixels names its tier beside it:
//!
//! - Tier 1 (geometry): positions, bounds and rectangles, within ±1 device
//!   px (a test may ask for less).
//! - Tier 2 (compositor): the result of a compositor op (blend, transform,
//!   filter, mask, backdrop, a solid fill), within ±2/255 per channel.
//! - Tier 3 (raster): pixels that Vello rasterises (anti-aliased outlines,
//!   glyphs, the blur of a box shadow, gradient lookup tables). They are
//!   compared with `Screenshot::raster_difference(..)` and
//!   `within_raster_tolerance()`: the mean of each pixel's largest channel
//!   difference is at most 1.0, at most 0.5% of the pixels differ by more
//!   than 64, and at least 99% differ by at most 8.

pub(crate) mod color_reference;
mod mount;
pub(crate) mod random;

pub(crate) use mount::{mount, mount_with};
