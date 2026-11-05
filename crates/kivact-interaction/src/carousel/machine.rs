//! Role: Own carousel navigation and the scroll settling timer.
use kivact_core::state::{Machine as MachineTrait, Step};
use std::time::Duration;
/// The current position or pending scroll destination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CarouselState {
    /// Settled on a slide.
    Idle { index: usize },
    /// Waiting for scrolling to stop.
    Scrolling {
        target_index: usize,
        settle_at_ns: u64,
    },
}
pub(super) enum Event {
    Go(usize),
    Scroll(usize, u64),
    Settle,
}
pub(super) enum Effect {
    Index(usize),
}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = CarouselState;
    type Event = Event;
    type Effect = Effect;
    fn transition(
        &self,
        state: &CarouselState,
        event: Event,
    ) -> Step<CarouselState, Event, Effect> {
        match event {
            Event::Go(index) => Step::to(CarouselState::Idle { index })
                .cancel(Event::Settle)
                .with(Effect::Index(index)),
            Event::Scroll(target_index, now) => Step::to(CarouselState::Scrolling {
                target_index,
                settle_at_ns: now.saturating_add(120_000_000),
            })
            .after(Duration::from_millis(120), Event::Settle),
            Event::Settle => match state {
                CarouselState::Scrolling { target_index, .. } => Step::to(CarouselState::Idle {
                    index: *target_index,
                })
                .with(Effect::Index(*target_index)),
                CarouselState::Idle { .. } => Step::stay(),
            },
        }
    }
}
