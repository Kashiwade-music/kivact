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
}

impl fmt::Display for ScalarRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFiniteBounds => "scalar bounds must be finite",
            Self::ReversedBounds => "scalar minimum exceeds maximum",
            Self::NonFiniteSpan => "scalar span cannot be represented as a finite value",
            Self::InvalidStep => "scalar step must be finite, positive and representable",
        })
    }
}

impl std::error::Error for ScalarRangeError {}

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
    pub fn new(min: f64, max: f64) -> Result<Self, ScalarRangeError> {
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
    pub fn with_step(mut self, step: f64) -> Result<Self, ScalarRangeError> {
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
    pub fn normalize(self, value: f64) -> Option<f64> {
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
    pub fn offset(self, value: f64, amount: f64) -> Option<f64> {
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
    pub fn step_by(self, value: f64, steps: f64) -> Option<f64> {
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
