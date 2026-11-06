//! Role: The ranges a scalar accepts, and the write-back when one leaves the value out.

use super::range_machine::{Effect, Ends, Event, Machine};
use crate::policy::scalar::ScalarRange;
use kivact_core::ErrorKind;
use kivact_core::custom::raise;
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Value, machine};
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

/// Connect a reactive range to a single atomic normalization transition.
#[track_caller]
pub(crate) fn follow_range(binding: Binding<f64>, range: Value<ScalarRange>) {
    let machine = machine(
        (),
        move || Machine {
            ends: Ends::One(binding.get_untracked()),
        },
        move |_, Effect::Write(_, value)| binding.set(value),
    );
    machine.feed(range, Event::RangeChanged);
}

/// Normalize both ends from the same snapshot, writing low before high.
#[track_caller]
pub(crate) fn follow_ranges(low: Binding<f64>, high: Binding<f64>, range: Value<ScalarRange>) {
    let machine = machine(
        (),
        move || Machine {
            ends: Ends::Two([low.get_untracked(), high.get_untracked()]),
        },
        move |_, Effect::Write(index, value)| {
            if index == 0 {
                low.set(value);
            } else {
                high.set(value);
            }
        },
    );
    machine.feed(range, Event::RangeChanged);
}

kivact_core::state::constant_value!(ScalarRange);
