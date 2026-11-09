//! Role: One paint layer: a color, gradient or image with its size, position and repeat.

use super::canvas_gradient::CanvasGradient;
use super::gradient::Gradient;
use crate::asset::Asset;
use crate::color::Color;
use crate::length::Length;
use crate::shader::data::ShaderData;
use crate::theme::value::ThemeColor;
use kivact_runtime::UiError;
use kivact_types::{BackgroundClip, BackgroundOrigin, BlendMode};

/// One fill: a color, a gradient or an image, with the size, position and
/// repetition of its tiles in the area it fills (one CSS background layer).
///
/// Sizes and positions are against the area filled (a node's border box,
/// or the rect a `Draw` command fills): `pct(50)` is half of it, and
/// `pct(100) - 20` 20 px less than all of it. By default a tile covers the
/// whole area (an image: its natural size) at its top left corner and
/// repeats along both axes, as in CSS.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{pct};
/// use kivact::draw::{Paint, linear_gradient};
/// // CSS `background: linear-gradient(90deg, #2563eb, #22d3ee) 0 0 / 40% 100% no-repeat`
/// let fill: Paint = linear_gradient(90)
///     .colors([rgb(0x2563eb), rgb(0x22d3ee)])
///     .size(pct(40), pct(100))
///     .no_repeat();
/// # let _ = fill;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    pub(crate) image: PaintImage,
    pub(crate) size: PaintSize,
    pub(crate) position: (Length, Length),
    // xtask-allow(state-fields): independent: the two axes of one authored repeat; every combination is valid
    pub(crate) repeat: (bool, bool),
    pub(crate) clip: BackgroundClip,
    // xtask-allow(state-fields): option: one authored property; absent means the inherited or default value
    pub(crate) origin: Option<BackgroundOrigin>,
    pub(crate) blend: BlendMode,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PaintImage {
    Color(Color),
    Gradient(Gradient),
    Canvas(CanvasGradient),
    Image(Asset),
    Shader(ShaderPaint),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PaintSize {
    Auto,
    Cover,
    Contain,
    Explicit(Length, Length),
}

impl Paint {
    fn of(image: PaintImage) -> Paint {
        Paint {
            image,
            size: PaintSize::Auto,
            position: (Length::Px(0.0), Length::Px(0.0)),
            repeat: (true, true),
            clip: BackgroundClip::BorderBox,
            origin: None,
            blend: BlendMode::Normal,
        }
    }

    /// The area the paint is clipped to (CSS `background-clip`; the
    /// default is the border box). `BackgroundClip::Text` paints only
    /// through the glyphs of a text node.
    pub fn clip(mut self, clip: BackgroundClip) -> Paint {
        self.clip = clip;
        self
    }

    /// The box the paint is positioned from (CSS `background-origin`;
    /// backgrounds default to the padding box, masks to the border box).
    pub fn origin(mut self, origin: BackgroundOrigin) -> Paint {
        self.origin = Some(origin);
        self
    }

    /// How the paint blends with the layers beneath it (CSS
    /// `background-blend-mode`).
    pub fn blend(mut self, mode: BlendMode) -> Paint {
        self.blend = mode;
        self
    }

    /// A paint shader fills the area (see [`ShaderPaint`]).
    pub(crate) fn from_shader(shader: ShaderPaint) -> Paint {
        Paint::of(PaintImage::Shader(shader))
    }

    /// The problems of the shader values this paint sets, and of the shaders
    /// in its fallback (for reporting).
    pub(crate) fn shader_problems(&self) -> Vec<UiError> {
        match &self.image {
            PaintImage::Shader(s) => {
                let mut out = s.problems().to_vec();
                if let Some(fallback) = &s.fallback {
                    out.extend(fallback.shader_problems());
                }
                out
            }
            _ => Vec::new(),
        }
    }

    /// An image (PNG, JPEG, WebP, SVG ..) at its natural size, repeated
    /// (CSS `background-image: url(..)`). It shows nothing until it has
    /// loaded; a failure is not reported (only `image(..)` nodes report
    /// `ErrorKind::Asset`).
    pub fn image(source: impl Into<Asset>) -> Paint {
        Paint::of(PaintImage::Image(source.into()))
    }

    /// The size of one tile (CSS `background-size: w h`); `auto()` keeps the
    /// natural size (an image: its aspect ratio from the other side; a
    /// gradient: the area's).
    pub fn size(mut self, width: impl Into<Length>, height: impl Into<Length>) -> Paint {
        self.size = PaintSize::Explicit(width.into(), height.into());
        self
    }

    /// Tiles as small as covers the area, keeping the aspect ratio (CSS
    /// `background-size: cover`).
    pub fn size_cover(mut self) -> Paint {
        self.size = PaintSize::Cover;
        self
    }

    /// Tiles as large as fits in the area, keeping the aspect ratio (CSS
    /// `background-size: contain`).
    pub fn size_contain(mut self) -> Paint {
        self.size = PaintSize::Contain;
        self
    }

    /// Where the first tile lies (CSS `background-position: x y`): a
    /// percentage puts that point of the tile on that point of the area
    /// (`pct(100)` aligns the tile's right or bottom edge with the area's).
    pub fn position(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Paint {
        self.position = (x.into(), y.into());
        self
    }

    /// One tile only (CSS `background-repeat: no-repeat`).
    pub fn no_repeat(mut self) -> Paint {
        self.repeat = (false, false);
        self
    }

    /// Repeat along x only (CSS `repeat-x`).
    pub fn repeat_x(mut self) -> Paint {
        self.repeat = (true, false);
        self
    }

    /// Repeat along y only (CSS `repeat-y`).
    pub fn repeat_y(mut self) -> Paint {
        self.repeat = (false, true);
        self
    }

    /// The color, when this is a color covering the area.
    pub(crate) fn solid(&self) -> Option<Color> {
        match self.image {
            PaintImage::Color(c)
                if self.size == PaintSize::Auto
                    && self.repeat == (true, true)
                    && self.clip == BackgroundClip::BorderBox
                    && self.origin.is_none()
                    && self.blend == BlendMode::Normal =>
            {
                Some(c)
            }
            _ => None,
        }
    }
}

impl<C: ThemeColor> From<C> for Paint {
    fn from(c: C) -> Self {
        Paint::of(PaintImage::Color(Color::of(c)))
    }
}

impl From<Gradient> for Paint {
    fn from(g: Gradient) -> Self {
        Paint::of(PaintImage::Gradient(g))
    }
}

impl From<CanvasGradient> for Paint {
    fn from(g: CanvasGradient) -> Self {
        Paint::of(PaintImage::Canvas(g))
    }
}

impl Gradient {
    /// As a [`Paint`] of tiles of this size ([`Paint::size`]).
    pub fn size(self, width: impl Into<Length>, height: impl Into<Length>) -> Paint {
        Paint::from(self).size(width, height)
    }

    /// As a [`Paint`] whose first tile is here ([`Paint::position`]).
    pub fn position(self, x: impl Into<Length>, y: impl Into<Length>) -> Paint {
        Paint::from(self).position(x, y)
    }

    /// As a [`Paint`] of one tile ([`Paint::no_repeat`]).
    pub fn no_repeat(self) -> Paint {
        Paint::from(self).no_repeat()
    }
}

/// A paint shader with its values: the parameters, the images for `image()`
/// and the clock. A [`Paint`] (`style().background(..)`, `mask(..)`).
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderPaint {
    pub(crate) data: ShaderData,
    pub(crate) fallback: Option<Box<Paint>>,
}

impl From<ShaderPaint> for Paint {
    fn from(s: ShaderPaint) -> Paint {
        Paint::from_shader(s)
    }
}
