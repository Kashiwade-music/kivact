//! Role: The ranges a scalar accepts, and the write-back when one leaves the value out.

use crate::policy::scalar::ScalarRange;
use kivact_core::ErrorKind;
use kivact_core::custom::raise;
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Value, effect_on};
use std::ops::RangeInclusive;
use std::panic::Location;

impl IntoValue<ScalarRange> for RangeInclusive<f64> {
    #[track_caller]
    fn into_prop(self) -> kivact_core::state::__private::Prop<ScalarRange> {
        let (min, max) = self.into_inner();
        let range = ScalarRange::new(min, max).unwrap_or_else(|error| {
            raise(
                ErrorKind::Message,
                format!("the range {min}..={max} is not valid: {error:?}"),
            )
        });
        kivact_core::state::__private::Prop::constant(range)
    }
}
impl IntoValue<ScalarRange> for RangeInclusive<i32> {
    #[track_caller]
    fn into_prop(self) -> kivact_core::state::__private::Prop<ScalarRange> {
        let (min, max) = self.into_inner();
        (f64::from(min)..=f64::from(max)).into_prop()
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

kivact_core::state::constant_value!(ScalarRange);
