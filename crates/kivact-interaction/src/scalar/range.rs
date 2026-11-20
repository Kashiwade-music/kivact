//! Role: The ranges a scalar accepts, and the write-back when one leaves the value out.

use crate::policy::scalar::{ScalarRange, ScalarStep};
use kivact_core::ErrorKind;
use kivact_core::custom::raise;
use kivact_core::state::IntoValue;
use kivact_core::state::{Signal, Value, effect_on};
use std::ops::RangeInclusive;

impl IntoValue<ScalarRange> for RangeInclusive<f64> {
    #[track_caller]
    fn into_prop(self) -> kivact_core::custom::Prop<ScalarRange> {
        let (min, max) = self.into_inner();
        let range = ScalarRange::new(min, max).unwrap_or_else(|error| {
            raise(
                ErrorKind::Message,
                format!("the range {min}..={max} is not valid: {error:?}"),
            )
        });
        kivact_core::custom::Prop::constant(range)
    }
}
impl IntoValue<ScalarRange> for RangeInclusive<i32> {
    #[track_caller]
    fn into_prop(self) -> kivact_core::custom::Prop<ScalarRange> {
        let (min, max) = self.into_inner();
        (f64::from(min)..=f64::from(max)).into_prop()
    }
}

impl IntoValue<ScalarStep> for f64 {
    #[track_caller]
    fn into_prop(self) -> kivact_core::custom::Prop<ScalarStep> {
        kivact_core::custom::Prop::constant(ScalarStep::Value(self))
    }
}

impl IntoValue<ScalarStep> for i32 {
    #[track_caller]
    fn into_prop(self) -> kivact_core::custom::Prop<ScalarStep> {
        kivact_core::custom::Prop::constant(ScalarStep::Value(f64::from(self)))
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
pub(crate) fn follow_range(binding: Signal<f64>, range: Value<ScalarRange>) {
    effect_on(range, move |range: &ScalarRange| {
        if let Some(value) = outside(*range, binding.get_untracked()) {
            // xtask-allow(effect-writes): a range prop change normalizes the caller's bound value into the new range (the base behavior of the range machine U08 removed); it writes only a value the range leaves out, so the next run writes nothing
            binding.set(value);
        }
    });
}
/// Normalize both ends from the same snapshot, writing low before high.
#[track_caller]
pub(crate) fn follow_ranges(low: Signal<f64>, high: Signal<f64>, range: Value<ScalarRange>) {
    effect_on(range, move |range: &ScalarRange| {
        let from = outside(*range, low.get_untracked());
        let to = outside(*range, high.get_untracked());
        if let Some(from) = from {
            // xtask-allow(effect-writes): a range prop change normalizes the caller's bound value into the new range (the base behavior of the range machine U08 removed); it writes only a value the range leaves out, so the next run writes nothing
            low.set(from);
        }
        if let Some(to) = to {
            // xtask-allow(effect-writes): a range prop change normalizes the caller's bound value into the new range (the base behavior of the range machine U08 removed); it writes only a value the range leaves out, so the next run writes nothing
            high.set(to);
        }
    });
}
