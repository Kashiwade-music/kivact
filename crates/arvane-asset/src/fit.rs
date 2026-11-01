//! Role: Where an image goes in its box: CSS `object-fit`.
//!
//! Invariants:
//! - The image is centered in the box for every fit.
//! - `Contain`, `Cover` and `ScaleDown` keep the natural aspect ratio;
//!   `Fill` stretches to the box; `None` keeps the natural size.
//! - `clip` is true exactly when the destination overflows the box (only
//!   `Cover` and `None` can), so the host clips to the box then.
//! - An empty natural size or box gives an empty destination at the box's
//!   center: nothing to draw.
//! - A [`Region`] is a part of an image in fractions of the rect it is
//!   drawn at; [`FitRect::visible`] is the part the box shows, so a host
//!   can ask for only that part of an SVG (an overflowing `Cover` or
//!   `None` image would otherwise be rasterized whole, up to the raster
//!   limits, and drawn blurry).

use ui_types::{Rect, Size};

pub use ui_types::ImageFit;

/// The destination of an image in its box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FitRect {
    /// Where to draw the whole image (may extend past the box).
    pub rect: Rect,
    /// Whether to clip the drawing to the box.
    pub clip: bool,
}

/// A part of an image, in fractions of the rect it is drawn at: `x` and
/// `y` from its left and top edges, `width` and `height` of its size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Region {
    /// Left edge, as a fraction of the drawn width.
    pub x: f32,
    /// Top edge, as a fraction of the drawn height.
    pub y: f32,
    /// Width, as a fraction of the drawn width.
    pub width: f32,
    /// Height, as a fraction of the drawn height.
    pub height: f32,
}

impl Region {
    /// All of the image.
    pub const WHOLE: Region = Region {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };

    /// The part of an image drawn at `rect` that `clip` lets through
    /// (empty when they do not overlap or `rect` is empty).
    pub fn visible(rect: Rect, clip: Rect) -> Region {
        let empty = Region {
            width: 0.0,
            height: 0.0,
            ..Region::WHOLE
        };
        if !(rect.width > 0.0 && rect.height > 0.0) {
            return empty;
        }
        let x0 = rect.x.max(clip.x);
        let y0 = rect.y.max(clip.y);
        let x1 = (rect.x + rect.width).min(clip.x + clip.width);
        let y1 = (rect.y + rect.height).min(clip.y + clip.height);
        if !(x1 > x0 && y1 > y0) {
            return empty;
        }
        Region {
            x: (x0 - rect.x) / rect.width,
            y: (y0 - rect.y) / rect.height,
            width: (x1 - x0) / rect.width,
            height: (y1 - y0) / rect.height,
        }
    }

    /// Where this part of an image drawn at `rect` goes.
    pub fn place(self, rect: Rect) -> Rect {
        Rect {
            x: rect.x + self.x * rect.width,
            y: rect.y + self.y * rect.height,
            width: self.width * rect.width,
            height: self.height * rect.height,
        }
    }
}

impl FitRect {
    /// The part of the image `bounds` (the box it was placed in) shows:
    /// all of it unless it overflows.
    pub fn visible(&self, bounds: Rect) -> Region {
        if self.clip {
            Region::visible(self.rect, bounds)
        } else {
            Region::WHOLE
        }
    }
}

/// Places an image of `natural` size in `bounds` by `fit` (see the module
/// invariants).
pub fn fit_image(fit: ImageFit, natural: Size, bounds: Rect) -> FitRect {
    let center = |width: f32, height: f32| Rect {
        x: bounds.x + (bounds.width - width) / 2.0,
        y: bounds.y + (bounds.height - height) / 2.0,
        width,
        height,
    };
    let usable = |v: f32| v.is_finite() && v > 0.0;
    if !usable(natural.width)
        || !usable(natural.height)
        || !usable(bounds.width)
        || !usable(bounds.height)
    {
        return FitRect {
            rect: center(0.0, 0.0),
            clip: false,
        };
    }
    let contain = (bounds.width / natural.width).min(bounds.height / natural.height);
    let scale = match fit {
        ImageFit::Fill => {
            return FitRect {
                rect: bounds,
                clip: false,
            };
        }
        ImageFit::Contain => contain,
        ImageFit::Cover => (bounds.width / natural.width).max(bounds.height / natural.height),
        ImageFit::None => 1.0,
        ImageFit::ScaleDown => contain.min(1.0),
    };
    let rect = center(natural.width * scale, natural.height * scale);
    // A rounding hair over the box is not an overflow.
    let eps = 1e-3;
    let clip = rect.width > bounds.width + eps || rect.height > bounds.height + eps;
    FitRect { rect, clip }
}
