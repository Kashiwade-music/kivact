//! Role: One delayed show and hide machine for tooltips, submenus and hover surfaces.
//!
//! [`Delayed`] follows a `wanted` condition late: it shows `open` after the
//! condition turns true and hides `close` after it turns false, on host-clock
//! task timers (no polling). [`Delayed::dismiss`] hides it at once and keeps it
//! hidden until the condition goes false, and [`Delayed::pulse`] shows it at
//! once and hides it after the close delay (a "Copied" mark).

use std::time::Duration;

use kivact_core::state::IntoValue;
use kivact_core::state::{Signal, Value, effect, memo, on_cleanup, signal, untrack};
use kivact_core::task::{Task, sleep, spawn};

/// How long a [`Delayed`] waits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Delays {
    /// From `wanted` becoming true to shown. Zero shows at once.
    pub open: Duration,
    /// From `wanted` becoming false to hidden. Zero hides at once.
    pub close: Duration,
}

/// Where a [`Delayed`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelayedPhase {
    /// Not wanted, not shown.
    Hidden,
    /// Wanted; the open delay runs.
    Opening,
    /// Shown.
    Shown,
    /// No longer wanted (or pulsed); still shown while the close delay runs.
    Closing,
    /// Hidden by `dismiss` while wanted: stays hidden until `wanted` goes false.
    Dismissed,
}

#[derive(Clone, Copy)]
enum State {
    Hidden,
    Opening(Task),
    Shown,
    Closing(Task),
    Dismissed,
}

/// A shown state that follows `wanted` late: shows `open` after it turns true
/// and hides `close` after it turns false, on host-clock task timers.
///
/// ```no_run
/// use std::time::Duration;
/// use kivact_core::{signal, control::{NodeBuilder, column, node, text}, view::show};
/// use kivact_interaction::{Delays, use_delayed};
/// let hovered = signal(false);
/// let card = use_delayed(hovered, Delays {
///     open: Duration::from_millis(300),
///     close: Duration::from_millis(100),
/// });
/// let view = column((
///     node().child(text("Hover me")),
///     show(card.shown(), move || node().child(text("The card"))),
/// ));
/// ```
#[derive(Clone, Copy)]
pub struct Delayed {
    state: Signal<State>,
    phase: Value<DelayedPhase>,
    shown: Value<bool>,
    wanted: Value<bool>,
    close: Duration,
}

/// Create a delayed state that follows `wanted`.
///
/// Nothing is shown until `wanted` is true (and the open delay passed). Setup
/// only.
#[track_caller]
pub fn use_delayed(wanted: impl IntoValue<bool>, delays: Delays) -> Delayed {
    delayed(wanted.into_value(), move || delays.open, delays.close)
}

/// A [`Delayed`] whose open delay is read at each false-to-true change of
/// `wanted` (the tooltip's skip delay). Setup only.
#[track_caller]
pub(crate) fn delayed(
    wanted: Value<bool>,
    open: impl Fn() -> Duration + 'static,
    close: Duration,
) -> Delayed {
    let state = signal(State::Hidden);
    let phase = memo(move || {
        state.with(|state| match state {
            State::Hidden => DelayedPhase::Hidden,
            State::Opening(_) => DelayedPhase::Opening,
            State::Shown => DelayedPhase::Shown,
            State::Closing(_) => DelayedPhase::Closing,
            State::Dismissed => DelayedPhase::Dismissed,
        })
    });
    let shown = memo(move || matches!(phase.get(), DelayedPhase::Shown | DelayedPhase::Closing));
    let this = Delayed {
        state,
        phase,
        shown,
        wanted,
        close,
    };
    effect(move || {
        let wanted = wanted.get();
        untrack(|| this.follow(wanted, &open));
    });
    // A timer spawned from a longer-lived owner's handler (`pulse`) must not
    // write the disposed state.
    on_cleanup(move || {
        if let State::Opening(task) | State::Closing(task) = state.with_untracked(|state| *state) {
            task.cancel();
        }
    });
    this
}

impl Delayed {
    /// Whether the content is shown: the phase is `Shown` or `Closing`
    /// (tracked).
    pub fn shown(self) -> Value<bool> {
        self.shown
    }

    /// Where the machine is (tracked).
    pub fn phase(self) -> Value<DelayedPhase> {
        self.phase
    }

    /// Hide now. While `wanted` is true it stays hidden (`Dismissed`) until
    /// `wanted` goes false; otherwise it is `Hidden`. Cancels a running delay.
    /// Call it from a handler or an effect.
    #[track_caller]
    pub fn dismiss(self) {
        let next = if self.wanted.get_untracked() {
            State::Dismissed
        } else {
            State::Hidden
        };
        self.enter(next);
    }

    /// Show now and hide after the close delay, as a release of `wanted`
    /// would: a confirmation that flashes after a press (a "Copied" mark).
    /// While `wanted` is true it is `Shown`. A pulse while `Closing` starts the
    /// close delay again. With a zero close delay it ends `Hidden` at once.
    /// Call it from a handler or an effect.
    #[track_caller]
    pub fn pulse(self) {
        if self.wanted.get_untracked() {
            self.enter(State::Shown);
        } else if self.close.is_zero() {
            self.enter(State::Hidden);
        } else {
            self.cancel_timer();
            let task = self.timer(self.close, State::Hidden);
            self.state.update(|state| *state = State::Closing(task));
        }
    }

    /// Move the machine for a change of `wanted`.
    #[track_caller]
    fn follow(self, wanted: bool, open: &dyn Fn() -> Duration) {
        let phase = self.phase.get_untracked();
        match (phase, wanted) {
            (DelayedPhase::Hidden, true) => {
                let delay = open();
                if delay.is_zero() {
                    self.enter(State::Shown);
                } else {
                    let task = self.timer(delay, State::Shown);
                    self.enter(State::Opening(task));
                }
            }
            (DelayedPhase::Opening, false) => self.enter(State::Hidden),
            (DelayedPhase::Shown, false) => {
                if self.close.is_zero() {
                    self.enter(State::Hidden);
                } else {
                    let task = self.timer(self.close, State::Hidden);
                    self.enter(State::Closing(task));
                }
            }
            (DelayedPhase::Closing, true) => self.enter(State::Shown),
            (DelayedPhase::Dismissed, false) => self.enter(State::Hidden),
            _ => {}
        }
    }

    /// Cancel the timer of the state being left and enter `next`.
    #[track_caller]
    fn enter(self, next: State) {
        self.cancel_timer();
        self.state.update(|state| *state = next);
    }

    /// Cancel the timer of the current state, if it has one.
    #[track_caller]
    fn cancel_timer(self) {
        if let State::Opening(task) | State::Closing(task) = self.state.with_untracked(|s| *s) {
            task.cancel();
        }
    }

    /// A task that enters `then` (`Shown` or `Hidden`) after `delay`. It does
    /// not cancel itself: every other transition out of `Opening` or `Closing`
    /// cancels it, so on completion the state is the one that started it.
    #[track_caller]
    fn timer(self, delay: Duration, then: State) -> Task {
        let state = self.state;
        spawn(async move {
            sleep(delay).await;
            state.update(|state| *state = then);
        })
    }
}
