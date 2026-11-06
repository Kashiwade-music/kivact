//! Role: Value scales, nice ticks and number formatting shared by the chart kinds.

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
        if !lo.is_finite() || !hi.is_finite() {
            (lo, hi) = (0.0, 1.0);
        }
        if hi - lo <= f64::EPSILON {
            hi = lo + 1.0;
        }
        let raw = (hi - lo) / 4.0;
        let magnitude = 10f64.powf(raw.log10().floor());
        let fraction = raw / magnitude;
        let step = magnitude
            * if fraction <= 1.0 {
                1.0
            } else if fraction <= 2.0 {
                2.0
            } else if fraction <= 5.0 {
                5.0
            } else {
                10.0
            };
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

/// `value` as JavaScript's `toLocaleString` writes it: grouped thousands, at most three decimals.
pub(super) fn format_value(value: f64) -> String {
    if !value.is_finite() {
        return String::from("-");
    }
    let fixed = format!("{:.3}", value.abs());
    let (whole, fraction) = fixed.split_once('.').unwrap_or((fixed.as_str(), ""));
    let fraction = fraction.trim_end_matches('0');
    let mut grouped = String::new();
    for (i, digit) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if !fraction.is_empty() {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    let negative = value < 0.0 && grouped.chars().any(|c| c.is_ascii_digit() && c != '0');
    if negative {
        format!("-{grouped}")
    } else {
        grouped
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
