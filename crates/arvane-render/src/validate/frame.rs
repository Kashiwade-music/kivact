//! Role: Decides a frame's pixel dimensions and raster scale within the frame budget.

use crate::error::RenderError;
use crate::limits::{MAX_PIXELS, MAX_SIDE};
use kivact_types::Size;

/// The pixel dimensions of a frame of logical `size` at `scale`, or why the
/// renderers do not draw it: the one place the frame budget is decided.
pub fn dimensions(size: Size, scale: f64) -> Result<(u16, u16), RenderError> {
    if !scale.is_finite()
        || scale <= 0.0
        || !size.width.is_finite()
        || !size.height.is_finite()
        || size.width <= 0.0
        || size.height <= 0.0
    {
        return Err(RenderError::InvalidSize);
    }
    let width = (size.width * scale).ceil();
    let height = (size.height * scale).ceil();
    if width < 1.0
        || height < 1.0
        || width > MAX_SIDE
        || height > MAX_SIDE
        || width * height > MAX_PIXELS
    {
        return Err(RenderError::InvalidSize);
    }
    Ok((width as u16, height as u16))
}

/// The logical extent and the physical scale to rasterize a `width` ×
/// `height` physical-pixel window at, whose display scale is `scale`: the
/// extent never rasterizes beyond the window, and a window beyond the
/// frame budget is drawn at a reduced scale (the frame is enlarged to the
/// window).
pub fn raster_plan(width: u32, height: u32, scale: f64) -> (Size, f64) {
    let size = raster_size(width, height, scale);
    let fits = |scale: f64| {
        let width = (size.width * scale).ceil();
        let height = (size.height * scale).ceil();
        width <= MAX_SIDE && height <= MAX_SIDE && width * height <= MAX_PIXELS
    };
    if !scale.is_finite() || scale <= 0.0 || fits(scale) {
        return (size, scale);
    }
    let (w, h) = (width as f64, height as f64);
    let mut reduced = scale
        * (MAX_SIDE / w)
            .min(MAX_SIDE / h)
            .min((MAX_PIXELS / (w * h)).sqrt());
    while !fits(reduced) {
        reduced *= 0.999;
    }
    (size, reduced)
}

/// Logical extent whose rasterization never exceeds the physical surface.
fn raster_size(width: u32, height: u32, scale: f64) -> Size {
    let extent = |physical: u32| {
        let mut logical = physical as f64 / scale;
        while logical * scale > physical as f64 {
            logical = logical.next_down();
        }
        logical
    };
    Size {
        width: extent(width),
        height: extent(height),
    }
}
