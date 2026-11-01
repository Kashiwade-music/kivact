//! Role: Conversion of paints into rasterizer brushes for the CPU and the GPU.
//!
//! Paints as rasterizer brushes: colors, gradients and patterns (the same
//! for the CPU and the GPU).
//!
//! Gradients: an elliptical radial gradient is the unit circle's,
//! stretched by the paint transform; two stops at the end offsets keep
//! their hard edge when padding (`gradient::stops_of`). Patterns: their
//! command count does not depend on how many tiles they cover. An image
//! tile of at most `RESAMPLE_MAX` (2048) device pixels a side is resampled
//! once (bilinear, edges clamped) to its device size and cached
//! (`derived.rs`), so both renderers repeat it pixel for pixel: the CPU as
//! a repeating image paint, the GPU from its image atlas or as one
//! textured rectangle per visible copy (`gpu/encode.rs`). A gradient tile
//! (2D: dots, checks) is rasterized on the CPU at its device size, at most
//! [`TILE_MAX`] a side, and repeated as an image; a 1D period is a
//! repeating gradient. Along an axis a pattern does not repeat along, it
//! is clipped to the band of its tile (`draw.rs`).
//!
//! Limits: a tile larger than `RESAMPLE_MAX` device pixels is drawn from
//! the image as it is, so the CPU (the whole image repeated with bilinear
//! sampling) and the GPU (each copy sampled apart, clamped at its edges)
//! differ slightly at the tiles' seams.

use crate::convert;
use crate::derived::Derived;
use crate::gradient::{color, gradient, stops_of};
use kivact_types::{ColorMatrix, ImageData, Paint, PatternSource, Rect};
use vello_cpu::PaintType;
use vello_cpu::kurbo::Affine;
use vello_cpu::peniko::Gradient as RasterGradient;

/// A paint ready to fill with.
pub(crate) enum Brush {
    /// A brush, painted through the paint transform when there is one
    /// (an elliptical radial gradient is a circle stretched).
    Paint(PaintType, Option<Affine>),
    /// Pixels repeated over the area.
    Pattern(PatternFill),
}

/// How [`crate::draw::Canvas::fill_pattern`] repeats pixels.
pub(crate) struct PatternFill {
    /// Straight-alpha pixels.
    pub(crate) image: ImageData,
    /// From the image's pixel grid to the current coordinates.
    pub(crate) transform: Affine,
    /// Repeat along x (else the edge pixels extend).
    pub(crate) repeat_x: bool,
    /// Repeat along y.
    pub(crate) repeat_y: bool,
    /// The area of one tile in the current coordinates.
    pub(crate) tile: Rect,
    /// One tile's size in device pixels.
    #[cfg_attr(
        not(feature = "gpu"),
        expect(dead_code, reason = "the GPU's atlas choice")
    )]
    pub(crate) device_tile: (f32, f32),
}

/// The brush of `paint` over an area of `bounds`, with every color through
/// `matrix`; `device_scale` is the device pixels per unit of the current
/// coordinates (gradient tiles are rasterized at it).
pub(crate) fn brush(
    paint: &Paint,
    bounds: Rect,
    matrix: Option<&ColorMatrix>,
    derived: &mut Derived,
    device_scale: f32,
) -> Brush {
    match paint {
        Paint::Solid(c) => Brush::Paint(PaintType::Solid(convert::color(color(*c, matrix))), None),
        Paint::LinearGradient { angle_deg, stops } => Brush::Paint(
            PaintType::Gradient(
                linear_angle(*angle_deg, bounds)
                    .with_stops(stops_of(stops, matrix, true).as_slice()),
            ),
            None,
        ),
        Paint::Gradient(g) => {
            let (paint, transform) = gradient(g, matrix);
            Brush::Paint(paint, transform)
        }
        Paint::Pattern(p) => {
            let device_tile = (p.tile.width * device_scale, p.tile.height * device_scale);
            let image = match &p.source {
                PatternSource::Image {
                    image,
                    color_matrix,
                } => {
                    let m = match (color_matrix, matrix) {
                        (Some(own), Some(m)) => Some(own.then(*m)),
                        (own, m) => own.or(m.copied()),
                    };
                    let image = match m {
                        Some(m) => derived.filtered(image, &m),
                        None => image.clone(),
                    };
                    // Repeated at its device size, pixel for pixel, the
                    // same on the CPU and the GPU (which samples each copy
                    // apart, clamped at its edges, as CSS does).
                    if device_tile.0 <= RESAMPLE_MAX && device_tile.1 <= RESAMPLE_MAX {
                        let size = |v: f32| v.round().max(1.0) as u16;
                        derived.resampled(&image, (size(device_tile.0), size(device_tile.1)))
                    } else {
                        image
                    }
                }
                PatternSource::Gradient(g) => {
                    let g = match matrix {
                        Some(m) => g.filtered(m),
                        None => g.clone(),
                    };
                    let size = |v: f32| v.round().clamp(1.0, f32::from(TILE_MAX)) as u16;
                    derived.gradient_tile(&g, p.tile, (size(device_tile.0), size(device_tile.1)))
                }
            };
            Brush::Pattern(PatternFill {
                transform: Affine::translate((f64::from(p.tile.x), f64::from(p.tile.y)))
                    * Affine::scale_non_uniform(
                        f64::from(p.tile.width) / f64::from(image.width),
                        f64::from(p.tile.height) / f64::from(image.height),
                    ),
                image,
                repeat_x: p.repeat_x,
                repeat_y: p.repeat_y,
                tile: p.tile,
                device_tile,
            })
        }
    }
}

/// The gradient line of [`Paint::LinearGradient`] (as
/// [`GradientShape::linear_angle`], in double precision).
fn linear_angle(angle_deg: f32, rect: Rect) -> RasterGradient {
    let angle = f64::from(angle_deg).to_radians();
    let (dy, dx) = angle.sin_cos();
    let span = f64::from(rect.width) * dx.abs() + f64::from(rect.height) * dy.abs();
    let cx = f64::from(rect.x) + f64::from(rect.width) / 2.0;
    let cy = f64::from(rect.y) + f64::from(rect.height) / 2.0;
    RasterGradient::new_linear(
        (cx - dx * span / 2.0, cy - dy * span / 2.0),
        (cx + dx * span / 2.0, cy + dy * span / 2.0),
    )
}

/// The longest side a gradient tile is rasterized at, in device pixels.
pub(crate) const TILE_MAX: u16 = 1024;

/// The longest side, in device pixels, of a pattern tile drawn from its
/// image resampled to the device size (16 MiB at most); a larger one is
/// drawn from the image as it is.
const RESAMPLE_MAX: f32 = 2048.0;
