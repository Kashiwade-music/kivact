//! Role: CSS `mask` layers: paints whose alpha (or luminance) masks what is
//! drawn beneath or inside them, combined layer by layer.

use super::fill_layer::fill_layer;
use super::gradient::Gradient;
use super::layer::Paint;
use crate::theme::value::{Theme, ThemeColor};
use kivact_types as p;
pub use kivact_types::{MaskComposite, MaskMode};

/// One CSS mask layer: a paint, whether it masks by alpha or luminance,
/// and how it combines with the layers beneath it.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::draw::{Mask, MaskComposite, MaskLayer, MaskMode, linear_gradient};
/// let fade = linear_gradient(180)
///     .colors([rgb(0xffffff), rgba(0xffffff, 0.0)]);
/// let m = Mask::from(MaskLayer::new(fade).mode(MaskMode::Luminance));
/// let s = style().mask(m.clone());
/// # let _ = (s, MaskComposite::Add);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MaskLayer {
    paint: Paint,
    mode: MaskMode,
    composite: MaskComposite,
}

impl MaskLayer {
    /// A mask layer of `paint` by alpha, added to the layers below.
    pub fn new(paint: impl Into<Paint>) -> MaskLayer {
        MaskLayer {
            paint: paint.into(),
            mode: MaskMode::Alpha,
            composite: MaskComposite::Add,
        }
    }

    /// Mask by alpha or by luminance (CSS `mask-mode`).
    pub fn mode(mut self, mode: MaskMode) -> MaskLayer {
        self.mode = mode;
        self
    }

    /// How this layer combines with the layers beneath it (CSS `mask-composite`).
    pub fn composite(mut self, op: MaskComposite) -> MaskLayer {
        self.composite = op;
        self
    }

    /// The paint of this layer.
    pub(crate) fn paint(&self) -> &Paint {
        &self.paint
    }

    /// The engine's layer, with its paint resolved against `theme`.
    pub(crate) fn wire(&self, theme: &Theme) -> p::MaskLayer {
        p::MaskLayer {
            fill: fill_layer(&self.paint, theme),
            mode: self.mode,
            composite: self.composite,
        }
    }
}

impl From<Paint> for MaskLayer {
    fn from(paint: Paint) -> Self {
        MaskLayer::new(paint)
    }
}

impl From<Gradient> for MaskLayer {
    fn from(g: Gradient) -> Self {
        MaskLayer::new(g)
    }
}

/// A CSS `mask`: its layers, the first on top.
#[derive(Clone, Debug, PartialEq)]
pub struct Mask(pub(crate) Vec<MaskLayer>);

impl Mask {
    /// The engine's layers, first on top.
    pub(crate) fn wire(&self, theme: &Theme) -> Vec<p::MaskLayer> {
        self.0.iter().map(|l| l.wire(theme)).collect()
    }
}

impl<C: ThemeColor> From<C> for Mask {
    fn from(c: C) -> Self {
        Mask(vec![MaskLayer::new(c)])
    }
}

impl From<Paint> for Mask {
    fn from(paint: Paint) -> Self {
        Mask(vec![MaskLayer::new(paint)])
    }
}

impl From<Gradient> for Mask {
    fn from(g: Gradient) -> Self {
        Mask(vec![MaskLayer::new(g)])
    }
}

impl From<MaskLayer> for Mask {
    fn from(layer: MaskLayer) -> Self {
        Mask(vec![layer])
    }
}

impl From<Vec<MaskLayer>> for Mask {
    fn from(layers: Vec<MaskLayer>) -> Self {
        Mask(layers)
    }
}

macro_rules! mask_tuple {
    ($($name:ident $t:ident),+) => {
        impl<$($t: Into<MaskLayer>),+> From<($($t,)+)> for Mask {
            fn from(($($name,)+): ($($t,)+)) -> Self {
                Mask(vec![$($name.into()),+])
            }
        }
    };
}

mask_tuple!(a A, b B);
mask_tuple!(a A, b B, c C);
mask_tuple!(a A, b B, c C, d D);
