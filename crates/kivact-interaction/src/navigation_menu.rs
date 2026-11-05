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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NavTimer {
    #[default]
    Idle,
    Opening(usize),
    Leaving(usize),
}
/// One owner records the visible panel and the actual scheduled command identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct NavMachineState {
    open: NavOpen,
    pending: NavTimer,
}
enum Event {
    Hover(usize),
    Leave(usize),
    Toggle(usize),
    Pin(usize),
    Close,
    HoverAfter(usize, Duration),
    LeaveAfter(usize, Duration),
    HoverDue(usize),
    LeaveDue(usize),
    PanelRemoved(usize),
}
enum Effect {
    Changed(bool),
}
struct Machine;
impl MachineTrait for Machine {
    type State = NavMachineState;
    type Event = Event;
    type Effect = Effect;
    fn transition(
        &self,
        state: &NavMachineState,
        event: Event,
    ) -> Step<NavMachineState, Event, Effect> {
        match event {
            Event::HoverAfter(key, delay) => schedule_open(*state, key, delay),
            Event::LeaveAfter(key, delay) => schedule_leave(*state, key, delay),
            Event::HoverDue(key) if state.pending == NavTimer::Opening(key) => enter(*state, key),
            Event::LeaveDue(key) if state.pending == NavTimer::Leaving(key) => leave(*state, key),
            Event::Hover(key) => enter(*state, key),
            Event::Leave(key) => leave(*state, key),
            Event::Toggle(key) => activate(
                *state,
                if state.open == NavOpen::Pinned(key) {
                    NavOpen::Closed
                } else {
                    NavOpen::Pinned(key)
                },
            ),
            Event::Pin(key) => activate(*state, NavOpen::Pinned(key)),
            Event::Close => activate(*state, NavOpen::Closed),
            Event::PanelRemoved(key) => remove_panel(*state, key),
            _ => Step::stay(),
        }
    }
}
fn publish(old: NavMachineState, next: NavMachineState) -> Step<NavMachineState, Event, Effect> {
    let step = Step::to(next);
    if matches!(old.open, NavOpen::Closed) != matches!(next.open, NavOpen::Closed) {
        step.with(Effect::Changed(!matches!(next.open, NavOpen::Closed)))
    } else {
        step
    }
}
fn cancel_timers(
    step: Step<NavMachineState, Event, Effect>,
) -> Step<NavMachineState, Event, Effect> {
    step.cancel(Event::HoverDue(0)).cancel(Event::LeaveDue(0))
}
fn activate(state: NavMachineState, open: NavOpen) -> Step<NavMachineState, Event, Effect> {
    cancel_timers(publish(
        state,
        NavMachineState {
            open,
            pending: NavTimer::Idle,
        },
    ))
}
fn enter(state: NavMachineState, key: usize) -> Step<NavMachineState, Event, Effect> {
    activate(
        state,
        if state.open == NavOpen::Pinned(key) {
            state.open
        } else {
            NavOpen::Hover(key)
        },
    )
}
fn leave(state: NavMachineState, key: usize) -> Step<NavMachineState, Event, Effect> {
    let open = if state.open == NavOpen::Hover(key) {
        NavOpen::Closed
    } else {
        state.open
    };
    let mut next = state;
    next.open = open;
    match state.pending {
        NavTimer::Opening(at) if at == key => {
            next.pending = NavTimer::Idle;
            if matches!(next.open, NavOpen::Hover(_)) {
                next.open = NavOpen::Closed;
            }
            publish(state, next).cancel(Event::HoverDue(0))
        }
        NavTimer::Leaving(at) if at == key => {
            next.pending = NavTimer::Idle;
            publish(state, next).cancel(Event::LeaveDue(0))
        }
        _ => publish(state, next),
    }
}
fn schedule_open(
    state: NavMachineState,
    key: usize,
    delay: Duration,
) -> Step<NavMachineState, Event, Effect> {
    if delay.is_zero() || matches!(state.open,NavOpen::Hover(at)|NavOpen::Pinned(at) if at==key) {
        return enter(state, key);
    }
    if state.pending == NavTimer::Opening(key) {
        return Step::stay();
    }
    cancel_timers(publish(
        state,
        NavMachineState {
            open: state.open,
            pending: NavTimer::Opening(key),
        },
    ))
    .after(delay, Event::HoverDue(key))
}
fn schedule_leave(
    state: NavMachineState,
    key: usize,
    delay: Duration,
) -> Step<NavMachineState, Event, Effect> {
    if delay.is_zero() || state.pending == NavTimer::Opening(key) {
        return leave(state, key);
    }
    if state.open != NavOpen::Hover(key) || state.pending == NavTimer::Leaving(key) {
        return Step::stay();
    }
    if matches!(state.pending, NavTimer::Opening(_)) {
        return leave(state, key);
    }
    cancel_timers(publish(
        state,
        NavMachineState {
            open: state.open,
            pending: NavTimer::Leaving(key),
        },
    ))
    .after(delay, Event::LeaveDue(key))
}
fn remove_panel(state: NavMachineState, key: usize) -> Step<NavMachineState, Event, Effect> {
    let mut next = state;
    if matches!(state.open,NavOpen::Hover(at)|NavOpen::Pinned(at) if at==key) {
        next.open = NavOpen::Closed;
    }
    match state.pending {
        NavTimer::Opening(at) if at == key => {
            next.pending = NavTimer::Idle;
            publish(state, next).cancel(Event::HoverDue(0))
        }
        NavTimer::Leaving(at) if at == key => {
            next.pending = NavTimer::Idle;
            publish(state, next).cancel(Event::LeaveDue(0))
        }
        _ => publish(state, next),
    }
}

/// Owner-scoped navigation visibility shared by all panels in a navigation menu.
#[derive(Clone, Copy)]
pub struct NavigationMenuApi {
    machine: MachineHandle<Machine>,
    changed: OptionsCell<Option<Callback<bool>>>,
    // xtask-allow(state-fields): Setup-only genuine panel identity allocation, not an event epoch or visibility cache.
    next_panel: OptionsCell<usize>,
}

/// Create one exclusive panel owner. Setup only.
#[track_caller]
pub fn use_navigation_menu() -> NavigationMenuApi {
    let changed = OptionsCell::new(None::<Callback<bool>>);
    NavigationMenuApi {
        machine: machine(
            NavMachineState::default(),
            || Machine,
            move |_, Effect::Changed(open)| {
                if let Some(changed) = changed.get() {
                    changed.call(open);
                }
            },
        ),
        changed,
        next_panel: OptionsCell::new(0),
    }
}
impl NavigationMenuApi {
    /// Allocate one genuine panel identity during setup, including mounted dynamic factories.
    /// This is identity allocation only; `NavOpen` remains the sole open-state authority.
    pub fn panel_key(self) -> usize {
        let key = self.next_panel.get();
        self.next_panel.update(|next| {
            *next = next
                .checked_add(1)
                .expect("navigation panel identity exhausted");
        });
        key
    }
    /// Observe accepted open/closed changes. Setup only.
    pub fn on_open_change(self, f: impl Fn(bool) + 'static) -> Self {
        self.changed.update(|changed| *changed = Some(callback(f)));
        self
    }
    /// The current panel and its opening mode.
    pub fn state(self) -> Value<NavOpen> {
        memo(move || self.machine.get().open)
    }
    /// Dispose this actual panel or its current eligibility without affecting another panel's command.
    pub fn panel_removed(self, key: usize) {
        self.machine.send(Event::PanelRemoved(key));
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
