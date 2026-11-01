//! Role: Layers, filters, masks, shadows and backdrops.
//! Plain data usable on any
//! thread.

use crate::{ColorMatrix, Paint, Point, Rect, Rgba, RoundedRect, Shape};

/// How a layer's pixels combine with what is beneath them (CSS
/// `mix-blend-mode`; [`BlendMode::PlusLighter`] is canvas `lighter`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    /// Source over.
    #[default]
    Normal,
    /// Multiply.
    Multiply,
    /// Screen.
    Screen,
    /// Overlay.
    Overlay,
    /// Darken.
    Darken,
    /// Lighten.
    Lighten,
    /// Color dodge.
    ColorDodge,
    /// Color burn.
    ColorBurn,
    /// Hard light.
    HardLight,
    /// Soft light.
    SoftLight,
    /// Difference.
    Difference,
    /// Exclusion.
    Exclusion,
    /// Hue.
    Hue,
    /// Saturation.
    Saturation,
    /// Color.
    Color,
    /// Luminosity.
    Luminosity,
    /// Additive (CSS `plus-lighter`, canvas `lighter`).
    PlusLighter,
}

/// A filter a layer's content goes through before it is clipped, masked
/// and composited.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerFilter {
    /// Gaussian blur (CSS `filter: blur(r)` is `std_dev = r`).
    Blur {
        /// Standard deviation in the layer's coordinates.
        std_dev: f32,
    },
    /// The content plus its blurred, offset silhouette beneath it in
    /// `color` (CSS `filter: drop-shadow(x y r color)` is
    /// `std_dev = r / 2`).
    DropShadow {
        /// Offset of the shadow.
        offset: Point,
        /// Standard deviation of its blur.
        std_dev: f32,
        /// Color (its alpha scales the silhouette's).
        color: Rgba,
    },
}

/// An alpha mask over a layer's content (CSS `mask-image` with a gradient
/// or an image): inside `rect`, the content keeps the alpha `paint` has
/// there; outside `rect` the content is hidden.
#[derive(Clone, Debug, PartialEq)]
pub struct Mask {
    /// Where the mask is.
    pub rect: Rect,
    /// Whose alpha masks (its color does not matter).
    pub paint: Paint,
}

/// A group of commands composited as one ([`crate::DrawCommand::PushLayer`]).
///
/// In CSS order: the content is drawn, goes through `filter`, is clipped
/// to `clip`, masked by `mask`, faded by `opacity` and blended with what is
/// beneath it by `blend`. A layer with only a clip costs a clip.
///
/// ```
/// use ui_types::{BlendMode, Layer};
/// let faded = Layer { opacity: 0.5, ..Layer::default() };
/// assert_eq!(faded.blend, BlendMode::Normal);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    /// Shape the result is clipped to.
    pub clip: Option<Shape>,
    /// Group opacity in `0.0..=1.0` (overlapping content does not get
    /// darker, unlike fading each command).
    pub opacity: f32,
    /// Blend with what is beneath.
    pub blend: BlendMode,
    /// Filter over the content.
    pub filter: Option<LayerFilter>,
    /// Alpha mask over the content.
    pub mask: Option<Mask>,
}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            clip: None,
            opacity: 1.0,
            blend: BlendMode::Normal,
            filter: None,
            mask: None,
        }
    }
}

/// A CSS `box-shadow` of a box with elliptical corners
/// ([`crate::DrawCommand::BoxShadow`]).
///
/// The shadow's shape is the box moved by `offset` and grown by `spread`
/// (inset: shrunk), its corner radii grown (inset: shrunk) as CSS says,
/// then blurred by a gaussian of `std_dev` (CSS blur radius / 2). An outer
/// shadow is drawn outside the box only when `knockout` is set (as CSS
/// draws it; unset, it is also drawn beneath the box, which costs less and
/// looks the same under an opaque background); an inset shadow is drawn
/// inside the box only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShadow {
    /// The box casting the shadow (its border box).
    pub shape: RoundedRect,
    /// Offset of the shadow.
    pub offset: Point,
    /// Growth of the shadow's shape (negative shrinks; inset shadows grow
    /// inwards).
    pub spread: f32,
    /// Standard deviation of the blur (0: a hard edge).
    pub std_dev: f32,
    /// Color.
    pub color: Rgba,
    /// Inside the box instead of outside.
    pub inset: bool,
    /// Outer shadows: leave the box itself out.
    pub knockout: bool,
}

/// What lies behind a shape, filtered (CSS `backdrop-filter`)
/// ([`crate::DrawCommand::Backdrop`]).
///
/// The renderer draws again the commands before this one that reach the
/// shape's bounds grown by the blur's reach, clipped to the shape, through
/// a blur of `std_dev` and with every color through `color_matrix`. It
/// reads no pixels back, so the CPU and the GPU agree. Limits: a backdrop
/// does not see what another backdrop drew (only the commands beneath
/// both); the color matrix applies to each replayed command's colors, which
/// equals applying it to their composite for matrices without offsets
/// (saturate, hue rotation, grayscale, sepia, brightness) and approximates
/// it otherwise; the cost is that of the replayed commands plus one blur
/// over the grown bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct Backdrop {
    /// Where the backdrop shows.
    pub shape: Shape,
    /// Standard deviation of the blur (0: none).
    pub std_dev: f32,
    /// Color matrix over the replayed colors (CSS `saturate(1.6)` ..).
    pub color_matrix: Option<ColorMatrix>,
}
