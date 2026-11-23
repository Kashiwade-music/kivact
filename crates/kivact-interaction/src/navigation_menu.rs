//! Role: Own hover and pinned navigation panel transitions.
use crate::options_cell::OptionsCell;
use crate::{PopupApi, PopupState, use_popup};
use kivact_core::custom::Part;
use kivact_core::state::{
    IntoValue, Machine as MachineTrait, MachineHandle, Signal, Step, Value, binding, machine, memo,
};
use kivact_core::view::{Element, IntoView, Placement, View, overlay, show};
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
    ShowFirst,
    /// The caller's `open` value changed; acts only when it differs from the machine's own open state.
    Sync(bool),
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
            Event::ShowFirst if state.open == NavOpen::Closed => {
                activate(*state, NavOpen::Pinned(0))
            }
            // The machine's own writes to `open` come back here equal to its state: ignore them,
            // so a stale echo never cancels another panel's pending timer.
            Event::Sync(true) if state.open == NavOpen::Closed => {
                activate(*state, NavOpen::Pinned(0))
            }
            Event::Sync(false) if state.open != NavOpen::Closed => {
                activate(*state, NavOpen::Closed)
            }
            Event::Sync(_) => Step::stay(),
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
/// Cancel only the timer of the pending command that the transition replaces.
fn cancel_timers(
    step: Step<NavMachineState, Event, Effect>,
    pending: NavTimer,
) -> Step<NavMachineState, Event, Effect> {
    match pending {
        NavTimer::Idle => step,
        NavTimer::Opening(key) => step.cancel(Event::HoverDue(key)),
        NavTimer::Leaving(key) => step.cancel(Event::LeaveDue(key)),
    }
}
fn activate(state: NavMachineState, open: NavOpen) -> Step<NavMachineState, Event, Effect> {
    cancel_timers(
        publish(
            state,
            NavMachineState {
                open,
                pending: NavTimer::Idle,
            },
        ),
        state.pending,
    )
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
            publish(state, next).cancel(Event::HoverDue(key))
        }
        NavTimer::Leaving(at) if at == key => {
            next.pending = NavTimer::Idle;
            publish(state, next).cancel(Event::LeaveDue(key))
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
    cancel_timers(
        publish(
            state,
            NavMachineState {
                open: state.open,
                pending: NavTimer::Opening(key),
            },
        ),
        state.pending,
    )
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
    cancel_timers(
        publish(
            state,
            NavMachineState {
                open: state.open,
                pending: NavTimer::Leaving(key),
            },
        ),
        state.pending,
    )
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
            publish(state, next).cancel(Event::HoverDue(key))
        }
        NavTimer::Leaving(at) if at == key => {
            next.pending = NavTimer::Idle;
            publish(state, next).cancel(Event::LeaveDue(key))
        }
        _ => publish(state, next),
    }
}

/// Owner-scoped navigation visibility shared by all panels in a navigation menu.
#[derive(Clone, Copy)]
pub struct NavigationMenuApi {
    machine: MachineHandle<Machine>,
    open: Signal<bool>,
    next_panel: OptionsCell<usize>,
}

/// Create one exclusive panel owner whose open state is `open`. Setup only.
#[track_caller]
pub fn use_navigation_menu(open: Signal<bool>) -> NavigationMenuApi {
    let machine = machine(
        NavMachineState::default(),
        || Machine,
        move |_, Effect::Changed(value)| {
            if open.get_untracked() != value {
                open.set(value);
            }
        },
    );
    machine.feed(open, Event::Sync);
    NavigationMenuApi {
        machine,
        open,
        next_panel: OptionsCell::new(0),
    }
}
impl NavigationMenuApi {
    /// Whether any panel is open.
    pub fn is_open(self) -> Value<bool> {
        self.open.into_value()
    }
    /// The current panel and its opening mode.
    pub fn state(self) -> Value<NavOpen> {
        memo(move || self.machine.get().open)
    }
    /// Show the first panel pinned. Nothing changes while a panel is open.
    pub fn show(self) {
        self.machine.send(Event::ShowFirst);
    }
    /// Hide the open panel, after dismissal or choosing a link.
    pub fn hide(self) {
        self.machine.send(Event::Close);
    }
    /// Show the first panel, or hide the open one.
    pub fn toggle(self) {
        if self.open.get_untracked() {
            self.hide();
        } else {
            self.show();
        }
    }
    /// Allocate one genuine panel and its API during setup, including mounted dynamic factories.
    #[track_caller]
    pub fn panel(self) -> NavigationPanelApi {
        let key = self.next_panel.get();
        self.next_panel.update(|next| {
            *next = next
                .checked_add(1)
                .expect("navigation panel identity exhausted");
        });
        let state = self.state();
        let pinned = binding(
            move || state.get() == NavOpen::Pinned(key),
            move |pin: bool| {
                if pin {
                    self.machine.send(Event::Pin(key));
                } else if state.get_untracked() == NavOpen::Pinned(key) {
                    self.machine.send(Event::Close);
                }
            },
        );
        let shown =
            memo(move || matches!(state.get(), NavOpen::Hover(i) | NavOpen::Pinned(i) if i == key));
        NavigationPanelApi {
            menu: self,
            key,
            shown,
            popup: use_popup(pinned),
        }
    }
}

/// One navigation panel of a menu: its trigger, its surface and its visibility.
#[derive(Clone, Copy)]
pub struct NavigationPanelApi {
    menu: NavigationMenuApi,
    key: usize,
    shown: Value<bool>,
    popup: PopupApi,
}

impl NavigationPanelApi {
    /// The panel's identity within its menu.
    pub fn key(self) -> usize {
        self.key
    }
    /// Whether this panel is shown, by hover or by pin.
    pub fn is_open(self) -> Value<bool> {
        self.shown
    }
    /// Pin this panel open: it stays open when the pointer leaves.
    pub fn show(self) {
        self.menu.machine.send(Event::Pin(self.key));
    }
    /// Close this panel if it is shown.
    pub fn hide(self) {
        if self.shown.get_untracked() {
            self.menu.machine.send(Event::Close);
        }
    }
    /// Pin this panel, or close it if it is pinned.
    pub fn toggle(self) {
        self.menu.machine.send(Event::Toggle(self.key));
    }
    /// Show this panel on pointer entry.
    pub fn hover(self) {
        self.menu.machine.send(Event::Hover(self.key));
    }
    /// End the pointer's visit to this panel.
    pub fn leave(self) {
        self.menu.machine.send(Event::Leave(self.key));
    }
    /// Dispose this panel's eligibility when it is removed.
    pub fn removed(self) {
        self.menu.machine.send(Event::PanelRemoved(self.key));
    }
    /// Schedule pointer opening; later input supersedes the pending timer.
    pub fn hover_after(self, delay: Duration) {
        self.menu.machine.send(Event::HoverAfter(self.key, delay));
    }
    /// Schedule pointer departure; re-entry supersedes the pending timer.
    pub fn leave_after(self, delay: Duration) {
        self.menu.machine.send(Event::LeaveAfter(self.key, delay));
    }
    /// The trigger: the popup trigger that also publishes `expanded` for this panel.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        let base = self.popup.trigger();
        Part::new(base.state(), move |host| {
            host.part(base).expanded(self.shown)
        })
    }
    /// The surface part, for the caller's `overlay(..)`.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        self.popup.content()
    }
    /// The panel's surface, mounted at the menu's anchor while the panel is shown.
    #[track_caller]
    pub fn surface<V: IntoView>(
        self,
        placement: Placement,
        content: impl Fn() -> V + 'static,
    ) -> View {
        show(self.shown, move || {
            overlay(self.popup.anchor_ref(), placement, content())
                .with_node(|node| node.part(self.content()))
        })
        .into_view()
    }
}
