//! Role: One delayed show and hide machine for tooltips, submenus and hover surfaces.
//!
//! [`DelayedApi`] follows a `wanted` condition late: it shows `open` after the
//! condition turns true and hides `close` after it turns false, on owner-scoped
//! machine timers (no polling). [`DelayedApi::hide`] hides it at once and keeps it
//! hidden until the condition goes false, and [`DelayedApi::pulse`] shows it at
//! once and hides it after the close delay (a "Copied" mark).

use std::time::Duration;

use kivact_core::state::IntoValue;
use kivact_core::state::{MachineHandle, Value, effect, machine, memo};

use super::machine::{Effect, Event, Machine, State};

/// How long a [`DelayedApi`] waits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Delays {
    /// From `wanted` becoming true to shown. Zero shows at once.
    pub open: Duration,
    /// From `wanted` becoming false to hidden. Zero hides at once.
    pub close: Duration,
}

/// Where a [`DelayedApi`] is.
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

/// A shown state that follows `wanted` late: shows `open` after it turns true
/// and hides `close` after it turns false, on owner-scoped machine timers.
///
/// ```no_run
/// use std::time::Duration;
/// use kivact_core::{state::signal, view::{Element, column, node, text, show}};
/// use kivact_interaction::{Delays, use_delayed};
/// let hovered = signal(false);
/// let card = use_delayed(hovered, Delays {
///     open: Duration::from_millis(300),
///     close: Duration::from_millis(100),
/// });
/// let view = column((
///     node().child(text("Hover me")),
///     show(card.is_open(), move || node().child(text("The card"))),
/// ));
/// ```
#[derive(Clone, Copy)]
pub struct DelayedApi {
    state: MachineHandle<Machine>,
    phase: Value<DelayedPhase>,
    shown: Value<bool>,
}

/// Create a delayed state that follows `wanted`.
///
/// Nothing is shown until `wanted` is true (and the open delay passed). Setup
/// only.
#[track_caller]
pub fn use_delayed(wanted: impl IntoValue<bool>, delays: Delays) -> DelayedApi {
    delayed(
        wanted.into_value(),
        move || delays.open,
        delays.close,
        |_| (),
    )
}

/// A [`DelayedApi`] whose open delay is read at each false-to-true change of
/// `wanted` (the tooltip's skip delay). Setup only.
#[track_caller]
pub(crate) fn delayed(
    wanted: Value<bool>,
    open: impl Fn() -> Duration + 'static,
    close: Duration,
    on_visible: impl Fn(bool) + 'static,
) -> DelayedApi {
    let state = machine(
        State::Hidden,
        move || Machine {
            wanted: wanted.get_untracked(),
            open: open(),
            close,
        },
        move |_, effect| match effect {
            Effect::Shown => on_visible(true),
            Effect::Hidden => on_visible(false),
        },
    );
    let phase = memo(move || match state.get() {
        State::Hidden => DelayedPhase::Hidden,
        State::Opening => DelayedPhase::Opening,
        State::Shown => DelayedPhase::Shown,
        State::Closing => DelayedPhase::Closing,
        State::Dismissed => DelayedPhase::Dismissed,
    });
    let shown = memo(move || matches!(phase.get(), DelayedPhase::Shown | DelayedPhase::Closing));
    let this = DelayedApi {
        state,
        phase,
        shown,
    };
    effect(move || state.send(Event::Wanted(wanted.get())));
    this
}

impl DelayedApi {
    /// Whether the content is shown: the phase is `Shown` or `Closing`
    /// (tracked).
    pub fn is_open(self) -> Value<bool> {
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
    pub fn hide(self) {
        self.state.send(Event::Dismiss);
    }

    /// Show now and hide after the close delay, as a release of `wanted`
    /// would: a confirmation that flashes after a press (a "Copied" mark).
    /// While `wanted` is true it is `Shown`. A pulse while `Closing` starts the
    /// close delay again. With a zero close delay it ends `Hidden` at once.
    /// Call it from a handler or an effect.
    #[track_caller]
    pub fn pulse(self) {
        self.state.send(Event::Pulse);
    }

    /// Dismiss whenever `requested` changes to `false`, as `dismiss` does; a
    /// change to `true` changes nothing. Setup only.
    pub(crate) fn dismiss_on_release(self, requested: impl IntoValue<bool>) {
        self.state.feed(requested, Event::Request);
    }

    /// End a running open delay at once whenever `immediate` changes to
    /// `true`, as `pulse` does while `Opening`. Setup only.
    pub(crate) fn hurry_on(self, immediate: impl IntoValue<bool>) {
        self.state.feed(immediate, Event::Immediate);
    }
}
