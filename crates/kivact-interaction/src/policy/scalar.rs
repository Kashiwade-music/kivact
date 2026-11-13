//! Role: Validated scalar bounds and optional quantization, independent of a UI runtime.

use std::fmt;

/// Invalid bounds or quantization for [`ScalarRange`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarRangeError {
    /// Both endpoints must be finite.
    NonFiniteBounds,
    /// The minimum must not exceed the maximum.
    ReversedBounds,
    /// The finite endpoints are too far apart to represent their span.
    NonFiniteSpan,
    /// A step must be finite, positive and representable over this range.
    InvalidStep,
    /// A mapping's exponent must be finite and positive.
    InvalidExponent,
}

impl fmt::Display for ScalarRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFiniteBounds => "scalar bounds must be finite",
            Self::ReversedBounds => "scalar minimum exceeds maximum",
            Self::NonFiniteSpan => "scalar span cannot be represented as a finite value",
            Self::InvalidStep => "scalar step must be finite, positive and representable",
            Self::InvalidExponent => "scalar mapping exponent must be finite and positive",
        })
    }
}

impl std::error::Error for ScalarRangeError {}

/// What one key press, wheel notch or assistive increment moves a scalar by.
///
/// A plain number is a [`Value`](Self::Value) step: `key_step(0.5)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScalarStep {
    /// In the value's units (1 dB, 1 frame). The range's own quantization still
    /// applies.
    Value(f64),
    /// A fraction of the control's travel, through its
    /// [`ScalarMapping`](crate::ScalarMapping): 0.01 is one percent of a
    /// logarithmic frequency knob wherever it is. A position step that the
    /// range's quantization would round back to the current value moves one
    /// quantization step instead.
    Position(f64),
}

impl ScalarStep {
    /// The amount this step moves by, in its own space.
    pub fn amount(self) -> f64 {
        match self {
            Self::Value(amount) | Self::Position(amount) => amount,
        }
    }
}

impl From<f64> for ScalarStep {
    fn from(amount: f64) -> Self {
        Self::Value(amount)
    }
}

impl From<i32> for ScalarStep {
    fn from(amount: i32) -> Self {
        Self::Value(f64::from(amount))
    }
}

/// Immutable, finite scalar bounds with optional quantization anchored at `min`.
///
/// `new` keeps ordered bounds explicit: reversed bounds are an error. Equal
/// endpoints are valid and always normalize to that endpoint. Quantization
/// rounds to the nearest step before clamping, so an endpoint need not be on
/// the step grid. This type stores neither a value nor a writer.
///
/// ```
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::ScalarRange;
/// let range = ScalarRange::new(0.0, 10.0)?.with_step(2.0)?;
/// assert_eq!(range.normalize(3.0), Some(4.0));
/// assert_eq!(range.normalize(100.0), Some(10.0));
/// assert_eq!(range.normalize(f64::NAN), None);
/// // Endpoints are not forced onto a step grid.
/// let uneven = ScalarRange::new(0.0, 9.0)?.with_step(4.0)?;
/// assert_eq!(uneven.normalize(9.0), Some(8.0));
/// # Ok::<(), kivact::interaction::ScalarRangeError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScalarRange {
    min: f64,
    max: f64,
    step: Option<f64>,
}

impl ScalarRange {
    /// Construct continuous bounds. No quantization is applied.
    pub fn new(min: impl Into<f64>, max: impl Into<f64>) -> Result<Self, ScalarRangeError> {
        let min = min.into();
        let max = max.into();
        if !min.is_finite() || !max.is_finite() {
            return Err(ScalarRangeError::NonFiniteBounds);
        }
        if min > max {
            return Err(ScalarRangeError::ReversedBounds);
        }
        if !(max - min).is_finite() {
            return Err(ScalarRangeError::NonFiniteSpan);
        }
        Ok(Self {
            min,
            max,
            step: None,
        })
    }

    /// Quantize every normalized value to this positive step before clamping.
    /// The min-anchored step count uses `f64::round`, including its tie policy.
    ///
    /// Keyboard increments can be configured separately by the facade's
    /// scalar control; a keyboard increment need not quantize pointer values.
    pub fn with_step(mut self, step: impl Into<f64>) -> Result<Self, ScalarRangeError> {
        let step = step.into();
        if !step.is_finite() || step <= 0.0 || !((self.max - self.min) / step).is_finite() {
            return Err(ScalarRangeError::InvalidStep);
        }
        self.step = Some(step);
        Ok(self)
    }

    /// The minimum endpoint.
    pub fn min(self) -> f64 {
        self.min
    }

    /// The maximum endpoint.
    pub fn max(self) -> f64 {
        self.max
    }

    /// The optional quantization step.
    pub fn step(self) -> Option<f64> {
        self.step
    }

    /// Reject non-finite input, otherwise quantize and clamp it.
    pub fn normalize(self, value: impl Into<f64>) -> Option<f64> {
        let value = value.into();
        if !value.is_finite() {
            return None;
        }
        if self.min == self.max {
            return Some(self.min);
        }
        let value = match self.step {
            Some(step) => {
                let offset = (value - self.min) / step;
                if !offset.is_finite() {
                    return Some(if value < self.min { self.min } else { self.max });
                }
                self.min + offset.round() * step
            }
            None => value,
        };
        Some(value.clamp(self.min, self.max))
    }

    /// Shift a normalized value by a finite amount, saturating at the bounds.
    pub fn offset(self, value: impl Into<f64>, amount: impl Into<f64>) -> Option<f64> {
        let value = value.into();
        let amount = amount.into();
        if !amount.is_finite() {
            return None;
        }
        let value = self.normalize(value)?;
        let next = value + amount;
        self.normalize(if next.is_finite() {
            next
        } else if amount > 0.0 {
            self.max
        } else {
            self.min
        })
    }

    /// Shift by step units (one unit when quantization is absent).
    ///
    /// Non-finite step counts are rejected. A finite count whose multiplication
    /// overflows saturates in its direction rather than writing infinity.
    pub fn step_by(self, value: impl Into<f64>, steps: impl Into<f64>) -> Option<f64> {
        let value = value.into();
        let steps = steps.into();
        if !steps.is_finite() {
            return None;
        }
        let amount = steps * self.step.unwrap_or(1.0);
        if amount.is_finite() {
            self.offset(value, amount)
        } else {
            self.normalize(value)?;
            self.normalize(if steps > 0.0 { self.max } else { self.min })
        }
    }
}
