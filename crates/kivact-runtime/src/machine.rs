//! Role: Run owner-scoped state machines with queued events, effects, and timers.
//!
//! A machine is like React's `useReducer` with effects and timers added.
//!
//! ```no_run
//! use std::time::Duration;
//! use kivact_runtime::{Machine, Step, machine};
//!
//! struct Counter;
//! enum Event { Increment, Reset }
//! impl Machine for Counter {
//!     type State = u32;
//!     type Event = Event;
//!     type Effect = u32;
//!     fn transition(&self, state: &u32, event: Event) -> Step<u32, Event, u32> {
//!         match event {
//!             Event::Increment => Step::to(state + 1).with(state + 1)
//!                 .after(Duration::from_secs(1), Event::Reset),
//!             Event::Reset => Step::to(0),
//!         }
//!     }
//! }
//! // Call during component setup, under a runtime owner.
//! let counter = machine(0, || Counter, |_, count| println!("{count}"));
//! counter.send(Event::Increment);
//! ```

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::mem::{Discriminant, discriminant};
use std::panic::Location;
use std::rc::Rc;
use std::time::Duration;

use crate::reactive::{
    IntoValue, Owned, Owner, Signal, Value, effect_on, require_effectful, untrack,
};
use crate::task::{Task, sleep, spawn_in};

/// A pure transition evaluated against a fresh input snapshot for each event.
pub trait Machine: Sized + 'static {
    /// The observable state of this machine.
    type State: crate::PropValue;
    /// An event accepted by this machine.
    type Event: 'static;
    /// A command executed after the state and timers have been updated.
    type Effect: 'static;
    /// Compute a transition without changing reactive state.
    fn transition(
        &self,
        state: &Self::State,
        event: Self::Event,
    ) -> Step<Self::State, Self::Event, Self::Effect>;
}

/// A transition's state change, ordered effects, and ordered timer operations.
pub struct Step<S, Ev, Ef> {
    next: Option<S>,
    effects: Vec<Ef>,
    timers: Vec<TimerOp<Ev>>,
}

enum TimerOp<Ev> {
    After(Duration, Ev),
    Cancel(Ev),
}

impl<S, Ev, Ef> Step<S, Ev, Ef> {
    /// Keep the current state.
    pub fn stay() -> Self {
        Self {
            next: None,
            effects: Vec::new(),
            timers: Vec::new(),
        }
    }
    /// Replace the current state.
    pub fn to(next: S) -> Self {
        Self {
            next: Some(next),
            ..Self::stay()
        }
    }
    /// Append an effect to run after this transition.
    pub fn with(mut self, effect: Ef) -> Self {
        self.effects.push(effect);
        self
    }
    /// Send `event` after `delay`, replacing a pending timer of the same event variant.
    pub fn after(mut self, delay: Duration, event: Ev) -> Self {
        self.timers.push(TimerOp::After(delay, event));
        self
    }
    /// Cancel the pending timer of `event`'s variant; its payload is ignored.
    pub fn cancel(mut self, event: Ev) -> Self {
        self.timers.push(TimerOp::Cancel(event));
        self
    }
}

enum Dispatch<Ev> {
    Idle,
    Running(VecDeque<Ev>),
}

struct MachineCell<M: Machine> {
    owner: Owner,
    state: Signal<M::State>,
    input: Box<dyn Fn() -> M>,
    run: Box<dyn Fn(MachineCx<M>, M::Effect)>,
    dispatch: RefCell<Dispatch<M::Event>>,
    timers: RefCell<HashMap<Discriminant<M::Event>, Task>>,
}

/// A copyable handle to an owner-scoped machine.
pub struct MachineHandle<M: Machine> {
    cell: Owned<Rc<MachineCell<M>>>,
}
impl<M: Machine> Clone for MachineHandle<M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<M: Machine> Copy for MachineHandle<M> {}

/// The context supplied to an effect for sending subsequent events.
pub struct MachineCx<M: Machine> {
    handle: MachineHandle<M>,
}
impl<M: Machine> MachineCx<M> {
    /// Queue an event after the current transition's effects.
    #[track_caller]
    pub fn send(&self, event: M::Event) {
        self.handle.send(event);
    }
}

/// Run `fire` after `delay` on a task of `owner`. Every machine's timers
/// share this one future type: the handle and the event are in `fire`.
#[inline(never)]
fn spawn_timer(
    owner: Owner,
    delay: Duration,
    fire: Box<dyn FnOnce()>,
    at: &'static Location<'static>,
) -> Task {
    let wait = sleep(delay);
    spawn_in(
        owner,
        async move {
            wait.await;
            fire();
        },
        at,
    )
}

/// Create a machine during setup. `input` builds each event's snapshot untracked.
#[track_caller]
pub fn machine<M: Machine>(
    initial: M::State,
    input: impl Fn() -> M + 'static,
    run: impl Fn(MachineCx<M>, M::Effect) + 'static,
) -> MachineHandle<M> {
    let at = Location::caller();
    let owner = Owner::current("machine", at);
    let state = owner.signal(initial, at);
    let cell = owner.store(
        "Machine",
        Rc::new(MachineCell {
            owner,
            state,
            input: Box::new(input),
            run: Box::new(run),
            dispatch: RefCell::new(Dispatch::Idle),
            timers: RefCell::new(HashMap::new()),
        }),
        at,
    );
    MachineHandle { cell }
}

impl<M: Machine> MachineHandle<M> {
    /// Dispatch now, or queue behind the currently running transition.
    #[track_caller]
    pub fn send(&self, event: M::Event) {
        let cell = self.cell.get();
        require_effectful("MachineHandle::send");
        {
            let mut dispatch = cell.dispatch.borrow_mut();
            match &mut *dispatch {
                Dispatch::Running(queue) => {
                    queue.push_back(event);
                    return;
                }
                Dispatch::Idle => *dispatch = Dispatch::Running(VecDeque::new()),
            }
        }
        let at = Location::caller();
        let mut current = event;
        loop {
            let step = untrack(|| (cell.input)().transition(&cell.state.get_untracked(), current));
            if let Some(next) = step.next {
                cell.state.set(next);
            }
            cell.owner.run_handler(|| {
                for timer in step.timers {
                    let (delay, event) = match timer {
                        TimerOp::After(delay, event) => (Some(delay), event),
                        TimerOp::Cancel(event) => (None, event),
                    };
                    let key = discriminant(&event);
                    if let Some(task) = cell.timers.borrow_mut().remove(&key) {
                        task.cancel();
                    }
                    if let Some(delay) = delay {
                        let handle = *self;
                        let fire: Box<dyn FnOnce()> = Box::new(move || handle.send(event));
                        let task = spawn_timer(cell.owner, delay, fire, at);
                        cell.timers.borrow_mut().insert(key, task);
                    }
                }
                for effect in step.effects {
                    (cell.run)(MachineCx { handle: *self }, effect);
                }
            });
            let mut dispatch = cell.dispatch.borrow_mut();
            match &mut *dispatch {
                Dispatch::Running(queue) => match queue.pop_front() {
                    Some(event) => current = event,
                    None => {
                        *dispatch = Dispatch::Idle;
                        return;
                    }
                },
                Dispatch::Idle => unreachable!("invariant: dispatch remains running until drained"),
            }
        }
    }
    /// The read-only reactive state.
    #[track_caller]
    pub fn state(&self) -> Value<M::State> {
        self.cell.get().state.into()
    }
    /// Read state and track it as a dependency.
    #[track_caller]
    pub fn get(&self) -> M::State {
        self.state().get()
    }
    /// Whenever `source` changes, send its new value as an event. Setup only; skips its initial value.
    #[track_caller]
    pub fn feed<T: crate::PropValue>(
        &self,
        source: impl IntoValue<T>,
        to_event: impl Fn(T) -> M::Event + 'static,
    ) {
        let handle = *self;
        effect_on(source, move |value| handle.send(to_event(value.clone())));
    }
}
