//! Role: Normalize all scalar ends for a range change before ordering binding writes.
use crate::policy::scalar::ScalarRange;
use kivact_core::state::{Machine as MachineTrait, Step};
#[derive(Clone, Copy)]
pub(super) enum Ends {
    One(f64),
    Two([f64; 2]),
}
pub(super) struct Machine {
    pub(super) ends: Ends,
}
pub(super) enum Event {
    RangeChanged(ScalarRange),
}
pub(super) enum Effect {
    Write(usize, f64),
}
impl MachineTrait for Machine {
    type State = ();
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, _: &(), Event::RangeChanged(range): Event) -> Step<(), Event, Effect> {
        let inside = |now: f64| {
            if now.is_finite() && (now < range.min() || now > range.max()) {
                range.normalize(now)
            } else {
                None
            }
        };
        let (from, to) = match self.ends {
            Ends::One(value) => (inside(value), None),
            Ends::Two([low, high]) => (inside(low), inside(high)),
        };
        let mut step = Step::stay();
        if let Some(from) = from {
            step = step.with(Effect::Write(0, from));
        }
        if let Some(to) = to {
            step = step.with(Effect::Write(1, to));
        }
        step
    }
}
