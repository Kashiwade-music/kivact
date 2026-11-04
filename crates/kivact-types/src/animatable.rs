//! Role: Component interpolation for scalar, tuple, and geometry values.

use crate::{Point, Rect, Size};

/// Values that can be interpolated component-wise.
pub trait Animatable: Clone + 'static {
    /// Number of f64 components.
    const COMPONENTS: usize;
    /// Append the `COMPONENTS` components of `self` to `out`.
    fn to_components(&self, out: &mut Vec<f64>);
    /// Build a value from `c` (missing components read as zero).
    fn from_components(c: &[f64]) -> Self;
}

impl Animatable for f32 {
    const COMPONENTS: usize = 1;
    fn to_components(&self, out: &mut Vec<f64>) {
        out.push(f64::from(*self));
    }
    fn from_components(components: &[f64]) -> Self {
        components.first().copied().unwrap_or_default() as f32
    }
}

impl Animatable for f64 {
    const COMPONENTS: usize = 1;
    fn to_components(&self, out: &mut Vec<f64>) {
        out.push(*self);
    }
    fn from_components(c: &[f64]) -> Self {
        c.first().copied().unwrap_or_default()
    }
}

macro_rules! tuple_animatable {
    ($($t:ident),+) => {
        impl<$($t: Animatable),+> Animatable for ($($t,)+) {
            const COMPONENTS: usize = 0 $(+ $t::COMPONENTS)+;
            #[allow(non_snake_case, reason = "tuple element bindings reuse the type parameter names")]
            fn to_components(&self, out: &mut Vec<f64>) {
                let ($($t,)+) = self;
                $($t.to_components(out);)+
            }
            #[allow(unused_assignments, reason = "the last element's offset update is never read")]
            fn from_components(c: &[f64]) -> Self {
                let mut i = 0;
                ($({
                    let end = (i + $t::COMPONENTS).min(c.len());
                    let v = $t::from_components(&c[i.min(end)..end]);
                    i += $t::COMPONENTS;
                    v
                },)+)
            }
        }
    };
}

tuple_animatable!(A, B);
tuple_animatable!(A, B, C);
tuple_animatable!(A, B, C, D);

impl Animatable for Point {
    const COMPONENTS: usize = 2;
    fn to_components(&self, out: &mut Vec<f64>) {
        out.extend_from_slice(&[self.x, self.y]);
    }
    fn from_components(c: &[f64]) -> Self {
        Point::new(
            c.first().copied().unwrap_or_default(),
            c.get(1).copied().unwrap_or_default(),
        )
    }
}

impl Animatable for Size {
    const COMPONENTS: usize = 2;
    fn to_components(&self, out: &mut Vec<f64>) {
        out.extend_from_slice(&[self.width, self.height]);
    }
    fn from_components(c: &[f64]) -> Self {
        Size::new(
            c.first().copied().unwrap_or_default(),
            c.get(1).copied().unwrap_or_default(),
        )
    }
}

impl Animatable for Rect {
    const COMPONENTS: usize = 4;
    fn to_components(&self, out: &mut Vec<f64>) {
        out.extend_from_slice(&[self.x, self.y, self.width, self.height]);
    }
    fn from_components(c: &[f64]) -> Self {
        let g = |i: usize| c.get(i).copied().unwrap_or_default();
        Rect::new(g(0), g(1), g(2), g(3))
    }
}
