//! Role: Provide binding-backed carousel commands and observable state.
use super::machine::{CarouselState, Effect, Event, Machine};
use kivact_core::state::{Binding, IntoBinding, IntoValue, MachineHandle, Value, machine, memo};
/// How scrolling settles between slides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Snap {
    /// Preserve the user's scroll position.
    None,
    /// Settle on the nearest slide after scrolling stops.
    #[default]
    Mandatory,
}
/// Headless carousel controller. Slide geometry belongs to its viewport.
#[derive(Clone, Copy)]
pub struct CarouselApi {
    state: MachineHandle<Machine>,
    index: Binding<usize>,
    count: usize,
}
/// Create a carousel controller over a caller-owned index and slide count.
#[track_caller]
pub fn use_carousel(index: impl IntoBinding<usize>, count: usize) -> CarouselApi {
    let index = index.into_binding();
    let state = machine(
        CarouselState::Idle {
            index: index.get_untracked().min(count.saturating_sub(1)),
        },
        || Machine,
        move |_, effect| match effect {
            Effect::Index(at) => index.set(at),
        },
    );
    state.feed(
        memo(move || index.get().min(count.saturating_sub(1))),
        Event::Go,
    );
    CarouselApi {
        state,
        index,
        count,
    }
}
impl CarouselApi {
    /// Observe the navigation state.
    pub fn state(self) -> Value<CarouselState> {
        memo(move || self.state.get())
    }
    /// Read the caller's selected slide.
    pub fn index(self) -> Value<usize> {
        self.index.into_value()
    }
    /// Select a slide, clamped to the available range.
    pub fn go_to(self, index: usize) {
        self.state
            .send(Event::Go(index.min(self.count.saturating_sub(1))));
    }
    /// Move by a signed number of slides, optionally wrapping.
    pub fn step(self, delta: isize, looping: bool) {
        if self.count == 0 {
            return;
        }
        let at = self.index.get_untracked() as isize;
        let count = self.count as isize;
        self.go_to(if looping {
            (at + delta).rem_euclid(count)
        } else {
            (at + delta).clamp(0, count - 1)
        } as usize);
    }
    /// Record the nearest slide during scrolling; commits after 120 ms of idle.
    pub fn scrolling(self, nearest: usize) {
        self.state.send(Event::Scroll(
            nearest.min(self.count.saturating_sub(1)),
            kivact_core::custom::clock_now(),
        ));
    }
}

kivact_core::state::constant_value!(Snap);
