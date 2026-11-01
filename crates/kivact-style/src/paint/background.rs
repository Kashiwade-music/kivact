//! Role: A box background: the stack of paint layers.

use super::gradient::Gradient;
use super::layer::Paint;
use crate::color::Color;
use crate::theme::value::ThemeColor;

/// A box's background: fills stacked like CSS background layers, the
/// first on top. A color, a gradient or a [`Paint`] is a background of one
/// layer; a tuple stacks several.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{Color, rgba};
/// use kivact::draw::linear_gradient;
/// // CSS `background: linear-gradient(#0002 1px, transparent 1px) 0 0 / 24px 24px,
/// //      linear-gradient(90deg, #0002 1px, transparent 1px) 0 0 / 24px 24px, white`
/// let line = rgba(0x000000, 0.12);
/// let grid = style().background((
///     linear_gradient(180).stop(line, 1).stop(Color::TRANSPARENT, 1).size(24, 24),
///     linear_gradient(90).stop(line, 1).stop(Color::TRANSPARENT, 1).size(24, 24),
///     Color::WHITE,
/// ));
/// # let _ = grid;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Background(pub(crate) Vec<Paint>);

impl Background {
    /// The color, when this is one color covering the box.
    pub(crate) fn solid(&self) -> Option<Color> {
        match self.0.as_slice() {
            [one] => one.solid(),
            _ => None,
        }
    }
}

impl From<Paint> for Background {
    fn from(p: Paint) -> Self {
        Background(vec![p])
    }
}

impl<C: ThemeColor> From<C> for Background {
    fn from(c: C) -> Self {
        Background::from(Paint::from(c))
    }
}

impl From<Gradient> for Background {
    fn from(g: Gradient) -> Self {
        Background::from(Paint::from(g))
    }
}

impl From<Vec<Paint>> for Background {
    fn from(v: Vec<Paint>) -> Self {
        Background(v)
    }
}

macro_rules! layers {
    ($($t:ident),+) => {
        impl<$($t: Into<Paint>),+> From<($($t,)+)> for Background {
            #[allow(non_snake_case, reason = "the tuple's elements are named by their types")]
            fn from(($($t,)+): ($($t,)+)) -> Self {
                Background(vec![$($t.into()),+])
            }
        }
    };
}

layers!(A, B);
layers!(A, B, C);
layers!(A, B, C, D);
layers!(A, B, C, D, E);
layers!(A, B, C, D, E, F);
