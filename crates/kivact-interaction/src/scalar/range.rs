//! Role: The ranges a scalar accepts, and the write-back when a range change leaves the value outside.

use crate::policy::scalar::ScalarRange;
use kivact_core::diagnostics::ErrorKind;
use kivact_core::primitive::raise;
use kivact_core::view::IntoValue;
use kivact_core::{Binding, Signal, Value, effect_on, memo};
use std::ops::RangeInclusive;
use std::panic::Location;

/// A range a [`Scalar`](super::Scalar) or [`SplitterApi`](crate::SplitterApi) reads: constant
/// bounds (`0.0..=100.0`), a constant [`ScalarRange`], or a closure, `Signal`
/// or `Value` that yields one and may change with the layout. Constant and
/// closure ranges need setup. The trait is sealed: those are its only forms.
pub trait IntoScalarRange: crate::sealed::IntoScalarRangeSeal {
    #[doc(hidden)]
    fn into_range(self) -> Value<ScalarRange>;
}

impl crate::sealed::IntoScalarRangeSeal for ScalarRange {}

impl crate::sealed::IntoScalarRangeSeal for RangeInclusive<f64> {}

impl crate::sealed::IntoScalarRangeSeal for Value<ScalarRange> {}

impl crate::sealed::IntoScalarRangeSeal for Signal<ScalarRange> {}

impl<F: Fn() -> ScalarRange + 'static> crate::sealed::IntoScalarRangeSeal for F {}

impl IntoScalarRange for ScalarRange {
    #[track_caller]
    fn into_range(self) -> Value<ScalarRange> {
        self.into_value()
    }
}

/// Continuous bounds from `min..=max`. Bounds that are not finite, or a
/// `min` above `max`, are a usage error located at the caller.
impl IntoScalarRange for RangeInclusive<f64> {
    #[track_caller]
    fn into_range(self) -> Value<ScalarRange> {
        let (min, max) = self.into_inner();
        match ScalarRange::new(min, max) {
            Ok(range) => range.into_value(),
            Err(error) => raise(
                ErrorKind::Message,
                format!("the range {min}..={max} is not valid: {error:?}"),
                Location::caller(),
            ),
        }
    }
}

impl IntoScalarRange for Value<ScalarRange> {
    fn into_range(self) -> Value<ScalarRange> {
        self
    }
}

impl IntoScalarRange for Signal<ScalarRange> {
    fn into_range(self) -> Value<ScalarRange> {
        self.into_value()
    }
}

impl<F: Fn() -> ScalarRange + 'static> IntoScalarRange for F {
    #[track_caller]
    fn into_range(self) -> Value<ScalarRange> {
        memo(self)
    }
}

/// The value a changed range moves `now` to: `None` while it is non-finite or
/// still inside the range.
pub(crate) fn pulled_inside(now: f64, range: ScalarRange) -> Option<f64> {
    if now.is_finite() && (now < range.min() || now > range.max()) {
        range.normalize(now)
    } else {
        None
    }
}

/// Write a value that a changed range left outside back inside it. A range
/// change that keeps the value inside writes nothing. One effect per scalar,
/// made where the scalar is. Setup only.
#[track_caller]
pub(crate) fn follow_range(binding: Binding<f64>, range: Value<ScalarRange>) {
    effect_on(range, move |range| {
        if let Some(inside) = pulled_inside(binding.get_untracked(), *range) {
            binding.set(inside);
        }
    });
}

kivact_core::view::constant_value!(ScalarRange);
