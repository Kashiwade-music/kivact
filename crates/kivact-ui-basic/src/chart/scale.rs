//! Role: Value scales and nice ticks of the chart kinds.

use crate::nice::round_step;

/// The ticks a value axis aims for: about four intervals, as the round step
/// of [`round_step`] counts them.
const TICKS: usize = 4;

/// A linear value axis from `lo` to `hi` in steps of `step`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Domain {
    pub(super) lo: f64,
    pub(super) hi: f64,
    pub(super) step: f64,
}

impl Domain {
    /// A domain that contains zero and `min`..`max`, ending on round tick values.
    pub(super) fn nice(min: f64, max: f64) -> Self {
        let (mut lo, mut hi) = (min.min(0.0), max.max(0.0));
        // Values that are not finite, or whose width overflows, get a unit range.
        if !(hi - lo).is_finite() {
            (lo, hi) = (0.0, 1.0);
        }
        if hi - lo <= f64::EPSILON {
            hi = lo + 1.0;
        }
        let step = round_step(lo, hi, TICKS).unwrap_or(hi - lo);
        Self {
            lo: (lo / step).floor() * step,
            hi: (hi / step).ceil() * step,
            step,
        }
    }

    /// The tick values from `lo` to `hi`.
    pub(super) fn ticks(&self) -> Vec<f64> {
        let count = ((self.hi - self.lo) / self.step).round() as usize;
        (0..=count)
            .map(|i| self.lo + self.step * i as f64)
            .collect()
    }

    /// Where `value` lies between `lo` (0) and `hi` (1).
    pub(super) fn fraction(&self, value: f64) -> f64 {
        (value - self.lo) / (self.hi - self.lo)
    }
}

/// A generous estimate of the width of `text` set at `size` px.
///
/// Hit testing runs outside the drawing closure, where text cannot be
/// measured, so axis gutters use this estimate in both places.
pub(super) fn text_width(text: &str, size: f64) -> f64 {
    text.chars()
        .map(|c| match c {
            '.' | ',' | ' ' | ':' | '\'' | 'i' | 'l' | 'j' | 't' | 'f' | 'I' => 0.36,
            'm' | 'w' | 'M' | 'W' => 0.9,
            c if c.is_ascii() => 0.62,
            _ => 1.0,
        })
        .sum::<f64>()
        * size
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the value domains of the charts.

    use super::*;

    #[test]
    fn a_domain_contains_zero_and_ends_on_multiples_of_its_step() {
        let high = Domain::nice(0.0, 305.0);
        assert_eq!((high.lo, high.hi, high.step), (0.0, 400.0, 100.0));
        let mixed = Domain::nice(-3.0, 9.0);
        assert_eq!((mixed.lo, mixed.hi, mixed.step), (-5.0, 10.0, 5.0));
        assert_eq!(mixed.ticks(), vec![-5.0, 0.0, 5.0, 10.0]);
    }

    #[test]
    fn an_empty_or_overflowing_range_still_has_a_finite_domain() {
        let empty = Domain::nice(0.0, 0.0);
        assert!(empty.step > 0.0 && empty.hi > empty.lo);
        let wide = Domain::nice(-f64::MAX, f64::MAX);
        assert_eq!((wide.lo, wide.hi), (0.0, 1.0));
    }
}
