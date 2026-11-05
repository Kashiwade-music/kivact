//! Role: Own hover and pinned navigation panel transitions.
use crate::options_cell::OptionsCell;
use kivact_core::state::{Callback, callback};
use kivact_core::state::{Machine as MachineTrait, MachineHandle, Step, Value, machine, memo};
use std::time::Duration;

/// The active navigation panel and the input that keeps it open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavOpen {
    #[default]
    Closed,
    Hover(usize),
    Pinned(usize),
}

enum Event {
    Hover(usize),
    Leave(usize),
    Toggle(usize),
    Pin(usize),
    Close,
    HoverAfter(usize, Duration),
    LeaveAfter(usize, Duration),
}
enum Effect {
    Changed(bool),
}
struct Machine;
impl MachineTrait for Machine {
    type State = NavOpen;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &NavOpen, event: Event) -> Step<NavOpen, Event, Effect> {
        let next = match event {
            Event::HoverAfter(at, delay) => {
                return if delay.is_zero() {
                    self.transition(state, Event::Hover(at))
                } else {
                    Step::stay()
                        .cancel(Event::Leave(at))
                        .after(delay, Event::Hover(at))
                };
            }
            Event::LeaveAfter(at, delay) => {
                return if delay.is_zero() {
                    self.transition(state, Event::Leave(at))
                } else {
                    Step::stay()
                        .cancel(Event::Hover(at))
                        .after(delay, Event::Leave(at))
                };
            }
            Event::Hover(at) if *state != NavOpen::Pinned(at) => NavOpen::Hover(at),
            Event::Leave(at) if *state == NavOpen::Hover(at) => NavOpen::Closed,
            Event::Toggle(at) if *state == NavOpen::Pinned(at) => NavOpen::Closed,
            Event::Toggle(at) | Event::Pin(at) => NavOpen::Pinned(at),
            Event::Close => NavOpen::Closed,
            Event::Hover(_) | Event::Leave(_) => *state,
        };
        let step = Step::to(next)
            .cancel(Event::Hover(0))
            .cancel(Event::Leave(0));
        if matches!(state, NavOpen::Closed) != matches!(next, NavOpen::Closed) {
            step.with(Effect::Changed(!matches!(next, NavOpen::Closed)))
        } else {
            step
        }
    }
}

/// Owner-scoped navigation visibility shared by all panels in a navigation menu.
#[derive(Clone, Copy)]
pub struct NavigationMenuApi {
    machine: MachineHandle<Machine>,
    changed: OptionsCell<Option<Callback<bool>>>,
}

/// Create one exclusive panel owner. Setup only.
#[track_caller]
pub fn use_navigation_menu() -> NavigationMenuApi {
    let changed = OptionsCell::new(None::<Callback<bool>>);
    NavigationMenuApi {
        machine: machine(
            NavOpen::Closed,
            || Machine,
            move |_, Effect::Changed(open)| {
                if let Some(changed) = changed.get() {
                    changed.call(open);
                }
            },
        ),
        changed,
    }
}
impl NavigationMenuApi {
    /// Observe accepted open/closed changes. Setup only.
    pub fn on_open_change(self, f: impl Fn(bool) + 'static) -> Self {
        self.changed.update(|changed| *changed = Some(callback(f)));
        self
    }
    /// The current panel and its opening mode.
    pub fn state(self) -> Value<NavOpen> {
        memo(move || self.machine.get())
    }
    /// Schedule pointer opening; later input supersedes the pending timer.
    pub fn hover_after(self, index: usize, delay: Duration) {
        self.machine.send(Event::HoverAfter(index, delay));
    }
    /// Schedule pointer departure; re-entry supersedes the pending timer.
    pub fn leave_after(self, index: usize, delay: Duration) {
        self.machine.send(Event::LeaveAfter(index, delay));
    }
    /// Enter a trigger or its hover surface.
    pub fn hover(self, index: usize) {
        self.machine.send(Event::Hover(index));
    }
    /// End the pointer's visit to a trigger and its surface.
    pub fn leave(self, index: usize) {
        self.machine.send(Event::Leave(index));
    }
    /// Pin a hovered panel, or close an already pinned panel.
    pub fn toggle(self, index: usize) {
        self.machine.send(Event::Toggle(index));
    }
    /// Keep a panel open after keyboard activation.
    pub fn pin(self, index: usize) {
        self.machine.send(Event::Pin(index));
    }
    /// Close after dismissal or choosing a link.
    pub fn close(self) {
        self.machine.send(Event::Close);
    }
}
