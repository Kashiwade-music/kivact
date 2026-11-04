//! Role: Drawing state shared by scalar controls and edit tracking.

use kivact_core::custom::PartState;

/// Reactive drawing state of an attached scalar thumb.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScalarState {
    pub input: PartState,
    /// The caller's current value.
    pub value: f64,
    /// Where the value lies in the current range, from 0 at the minimum to 1
    /// at the maximum. A value outside the range is clamped; an empty range or
    /// a non-finite value is 0.
    pub fraction: f64,
}
