//! Role: Map scalar values to positions along a control and back.

use crate::policy::scalar::{ScalarRange, ScalarRangeError};
use std::fmt;
use std::rc::Rc;

/// How a scalar's value maps to a position along its control, from 0 at the
/// range's minimum to 1 at its maximum: the curve a drag, a touch, a position
/// step and the drawing follow.
///
/// The value stays in the caller's units; only movement and drawing use the
/// position. A position is clamped to 0..=1 and the value a position maps to is
/// normalized by the range (clamped and quantized by its step). `to_position`
/// must increase with the value and `to_value` must be its inverse.
/// [`linear`](Self::linear) is the default.
///
/// ```
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{ScalarMapping, ScalarRange};
/// let hz = ScalarRange::new(20.0, 20_000.0)?;
/// let log = ScalarMapping::logarithmic();
/// assert!((log.position(632.455_532, hz) - 0.5).abs() < 1e-6);
/// assert_eq!(log.value(0.0, hz), Some(20.0));
/// let fader = ScalarMapping::skewed(0.5)?; // position = fraction^0.5
/// assert_eq!(fader.position(0.25, ScalarRange::new(0.0, 1.0)?), 0.5);
/// # Ok::<(), kivact::interaction::ScalarRangeError>(())
/// ```
#[derive(Clone)]
pub struct ScalarMapping {
    curve: Curve,
}

#[derive(Clone)]
enum Curve {
    Linear,
    Logarithmic,
    /// A validated finite exponent greater than zero and different from one.
    Skewed(f64),
    Custom {
        to_position: Rc<dyn Fn(f64, ScalarRange) -> f64>,
        to_value: Rc<dyn Fn(f64, ScalarRange) -> f64>,
    },
}

impl ScalarMapping {
    /// Equal value changes for equal travel.
    pub fn linear() -> Self {
        Self {
            curve: Curve::Linear,
        }
    }

    /// Equal ratios for equal travel (frequencies, times). Linear when the
    /// range's minimum is not positive or the range is empty.
    pub fn logarithmic() -> Self {
        Self {
            curve: Curve::Logarithmic,
        }
    }

    /// `position = fraction.powf(exponent)`, the skew of JUCE's
    /// `NormalisableRange` and nih-plug's `FloatRange::Skewed`: below 1 the low
    /// end gets more travel (a fader's 0.5), above 1 the high end. `1.0` is
    /// [`linear`](Self::linear). A non-finite or non-positive exponent is
    /// [`ScalarRangeError::InvalidExponent`].
    pub fn skewed(exponent: impl Into<f64>) -> Result<Self, ScalarRangeError> {
        let exponent = exponent.into();
        if !exponent.is_finite() || exponent <= 0.0 {
            return Err(ScalarRangeError::InvalidExponent);
        }
        let curve = if exponent == 1.0 {
            Curve::Linear
        } else {
            Curve::Skewed(exponent)
        };
        Ok(Self { curve })
    }

    /// Any curve, for example a plugin parameter's own normalization. Both
    /// functions receive the current range. A non-finite position counts as 0;
    /// a non-finite value is not written.
    pub fn custom(
        to_position: impl Fn(f64, ScalarRange) -> f64 + 'static,
        to_value: impl Fn(f64, ScalarRange) -> f64 + 'static,
    ) -> Self {
        Self {
            curve: Curve::Custom {
                to_position: Rc::new(to_position),
                to_value: Rc::new(to_value),
            },
        }
    }

    /// Where `value` lies along the control, clamped to 0..=1; 0 for a
    /// non-finite value or an empty range.
    pub fn position(&self, value: impl Into<f64>, range: ScalarRange) -> f64 {
        let value = value.into();
        let span = range.max() - range.min();
        if !value.is_finite() || span <= 0.0 {
            return 0.0;
        }
        let fraction = ((value - range.min()) / span).clamp(0.0, 1.0);
        let position = match &self.curve {
            Curve::Linear => fraction,
            Curve::Logarithmic if range.min() > 0.0 => {
                let min = range.min();
                (value.max(min) / min).ln() / (range.max() / min).ln()
            }
            Curve::Logarithmic => fraction,
            Curve::Skewed(exponent) => fraction.powf(*exponent),
            Curve::Custom { to_position, .. } => to_position(value, range),
        };
        if position.is_finite() {
            position.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The normalized value at `position` (clamped to 0..=1 first); `None` when
    /// the curve gives a non-finite value or `position` is not finite.
    pub fn value(&self, position: impl Into<f64>, range: ScalarRange) -> Option<f64> {
        let position = position.into();
        if !position.is_finite() {
            return None;
        }
        let position = position.clamp(0.0, 1.0);
        if range.min() == range.max() {
            return Some(range.min());
        }
        let span = range.max() - range.min();
        let value = match &self.curve {
            Curve::Linear => range.min() + position * span,
            Curve::Logarithmic if range.min() > 0.0 => {
                range.min() * (range.max() / range.min()).powf(position)
            }
            Curve::Logarithmic => range.min() + position * span,
            Curve::Skewed(exponent) => range.min() + span * position.powf(1.0 / exponent),
            Curve::Custom { to_value, .. } => to_value(position, range),
        };
        if !value.is_finite() {
            return None;
        }
        range.normalize(value)
    }

    /// Whether this is the linear mapping (a body drag then keeps the ends'
    /// value distance exactly).
    pub fn is_linear(&self) -> bool {
        matches!(self.curve, Curve::Linear)
    }
}

impl Default for ScalarMapping {
    fn default() -> Self {
        Self::linear()
    }
}

impl PartialEq for ScalarMapping {
    fn eq(&self, other: &Self) -> bool {
        match (&self.curve, &other.curve) {
            (Curve::Linear, Curve::Linear) => true,
            (Curve::Logarithmic, Curve::Logarithmic) => true,
            (Curve::Skewed(a), Curve::Skewed(b)) => a == b,
            (
                Curve::Custom {
                    to_position: a,
                    to_value: b,
                },
                Curve::Custom {
                    to_position: c,
                    to_value: d,
                },
            ) => Rc::ptr_eq(a, c) && Rc::ptr_eq(b, d),
            _ => false,
        }
    }
}

impl fmt::Debug for ScalarMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.curve {
            Curve::Linear => f.write_str("Linear"),
            Curve::Logarithmic => f.write_str("Logarithmic"),
            Curve::Skewed(exponent) => write!(f, "Skewed({exponent})"),
            Curve::Custom { .. } => f.write_str("Custom"),
        }
    }
}
