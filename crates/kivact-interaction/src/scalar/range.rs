//! Role: The ranges a scalar accepts, and the write-back when one leaves the value out.

use crate::policy::scalar::{ScalarRange, ScalarStep};
use kivact_core::ErrorKind;
use kivact_core::custom::raise;
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Value, effect_on};
use std::ops::RangeInclusive;

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

impl IntoValue<ScalarStep> for f64 {
    #[track_caller]
    fn into_prop(self) -> kivact_core::state::__private::Prop<ScalarStep> {
        kivact_core::state::__private::Prop::constant(ScalarStep::Value(self))
    }
}

impl IntoValue<ScalarStep> for i32 {
    #[track_caller]
    fn into_prop(self) -> kivact_core::state::__private::Prop<ScalarStep> {
        kivact_core::state::__private::Prop::constant(ScalarStep::Value(f64::from(self)))
    }
}

/// The value `now` takes in `range` when the range leaves it out: a finite
/// value outside the ends, normalized. Nothing for a value inside or not finite.
fn outside(range: ScalarRange, now: f64) -> Option<f64> {
    if now.is_finite() && (now < range.min() || now > range.max()) {
        range.normalize(now)
    } else {
        None
    }
}
/// Write the value back when a range change leaves it out.
#[track_caller]
pub(crate) fn follow_range(binding: Binding<f64>, range: Value<ScalarRange>) {
    effect_on(range, move |range: &ScalarRange| {
        if let Some(value) = outside(*range, binding.get_untracked()) {
            binding.set(value);
        }
    });
}
/// Normalize both ends from the same snapshot, writing low before high.
#[track_caller]
pub(crate) fn follow_ranges(low: Binding<f64>, high: Binding<f64>, range: Value<ScalarRange>) {
    effect_on(range, move |range: &ScalarRange| {
        let from = outside(*range, low.get_untracked());
        let to = outside(*range, high.get_untracked());
        if let Some(from) = from {
            low.set(from);
        }
        if let Some(to) = to {
            high.set(to);
        }
    });
}

kivact_core::state::constant_value!(ScalarRange);
kivact_core::state::constant_value!(ScalarStep);
