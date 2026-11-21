//! Role: CSS `border-image` (and the nine-slice of `Draw::border_image`): an
//! image or gradient cut into nine pieces that fill a border.

use super::fill_layer::fill_layer;
use super::gradient::Gradient;
use crate::asset::Asset;
use crate::resolve::Resolve;
use kivact_types as p;
use kivact_types::{BorderImageRepeat, Edges};

/// What a [`BorderImage`] paints: a picture or a gradient.
#[derive(Clone, Debug, PartialEq)]
pub enum BorderImageSource {
    /// A picture.
    Image(Asset),
    /// A CSS gradient.
    Gradient(Box<Gradient>),
}

impl From<Asset> for BorderImageSource {
    fn from(a: Asset) -> Self {
        BorderImageSource::Image(a)
    }
}

impl<const N: usize> From<&'static [u8; N]> for BorderImageSource {
    fn from(bytes: &'static [u8; N]) -> Self {
        BorderImageSource::Image(Asset::from(&bytes[..]))
    }
}

impl From<&'static str> for BorderImageSource {
    fn from(path: &'static str) -> Self {
        BorderImageSource::Image(Asset::from(path))
    }
}

impl From<Gradient> for BorderImageSource {
    fn from(g: Gradient) -> Self {
        BorderImageSource::Gradient(Box::new(g))
    }
}

/// A CSS border image: `border-image-source`, `-slice`, `-width`, `-outset`,
/// `-repeat` and whether the middle is filled.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{BorderImage, BorderImageRepeat};
/// // A 12 px frame cut from the picture, its edges repeated.
/// let frame = BorderImage::new(&[0u8; 4])
///     .slice(12)
///     .repeat(BorderImageRepeat::Round);
/// let s = style().border_image(frame);
/// # let _ = s;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct BorderImage {
    source: BorderImageSource,
    slice: Edges<f64>,
    slice_pct: bool,
    fill: bool,
    width: Option<Edges<f64>>,
    outset: Edges<f64>,
    repeat: [BorderImageRepeat; 2],
}

impl BorderImage {
    /// A border image of `source` (a picture or a gradient), sliced at 0.
    pub fn new(source: impl Into<BorderImageSource>) -> BorderImage {
        BorderImage {
            source: source.into(),
            slice: Edges::<f64>::default(),
            slice_pct: false,
            fill: false,
            width: None,
            outset: Edges::<f64>::default(),
            repeat: [BorderImageRepeat::Stretch; 2],
        }
    }

    /// The slices, in px of the source.
    pub fn slice(mut self, slice: impl Into<Edges<f64>>) -> BorderImage {
        self.slice = slice.into();
        self.slice_pct = false;
        self
    }

    /// The slices, in percent of the source.
    pub fn slice_pct(mut self, slice: impl Into<Edges<f64>>) -> BorderImage {
        self.slice = slice.into();
        self.slice_pct = true;
        self
    }

    /// Paint the middle of the source too (CSS `border-image-slice: fill`).
    pub fn fill(mut self) -> BorderImage {
        self.fill = true;
        self
    }

    /// The width of the border image area, in px (default: the border
    /// widths).
    pub fn width(mut self, width: impl Into<Edges<f64>>) -> BorderImage {
        self.width = Some(width.into());
        self
    }

    /// How far the border image reaches beyond the border box, in px.
    pub fn outset(mut self, outset: impl Into<Edges<f64>>) -> BorderImage {
        self.outset = outset.into();
        self
    }

    /// The same repetition on both axes.
    pub fn repeat(self, repeat: BorderImageRepeat) -> BorderImage {
        self.repeat_xy(repeat, repeat)
    }

    /// Repetition along x (`horizontal`) and y (`vertical`).
    pub fn repeat_xy(
        mut self,
        horizontal: BorderImageRepeat,
        vertical: BorderImageRepeat,
    ) -> BorderImage {
        self.repeat = [horizontal, vertical];
        self
    }

    /// The engine's border image, with its source resolved in `cx`.
    pub(crate) fn wire(&self, cx: &mut Resolve<'_>) -> p::BorderImage {
        let edges = |i: Edges<f64>, pct: bool| {
            let l = |v: f64| {
                if pct { p::Len::pct(v) } else { p::Len::px(v) }
            };
            p::Edges {
                top: l(i.top),
                right: l(i.right),
                bottom: l(i.bottom),
                left: l(i.left),
            }
        };
        let source = match &self.source {
            BorderImageSource::Image(a) => p::FillImage::Image(a.clone()),
            BorderImageSource::Gradient(g) => {
                let layer = fill_layer(&super::Paint::from(g.as_ref().clone()), cx);
                layer.image
            }
        };
        p::BorderImage {
            source,
            slice: edges(self.slice, self.slice_pct),
            fill: self.fill,
            width: self.width.map(|w| edges(w, false)),
            outset: p::Edges {
                top: self.outset.top,
                right: self.outset.right,
                bottom: self.outset.bottom,
                left: self.outset.left,
            },
            repeat: self.repeat,
        }
    }
}
