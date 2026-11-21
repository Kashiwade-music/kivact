//! Role: A latch that turns true once a node has been inside its window enough.

use super::NodeRef;
use kivact_runtime::{Machine, Step, Value, machine, memo};

/// Whether the node has been seen.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Seen {
    Waiting,
    Seen,
}

struct Latch;

impl Machine for Latch {
    type State = Seen;
    /// Whether the node is visible enough now.
    type Event = bool;
    type Effect = ();

    fn transition(&self, state: &Seen, visible: bool) -> Step<Seen, bool, ()> {
        match state {
            Seen::Waiting if visible => Step::to(Seen::Seen),
            _ => Step::stay(),
        }
    }
}

impl NodeRef {
    /// Become true once at least `fraction` (0 to 1) of the bound node's
    /// height is inside its window, and stay true: a reveal-on-scroll latch.
    #[track_caller]
    pub fn once_visible(&self, fraction: f64) -> Value<bool> {
        let rect = self.rect();
        let window = crate::platform::window_size();
        let visible = memo(move || {
            rect.get().is_some_and(|rect| {
                let shown = (rect.y + rect.height).min(window.get().height) - rect.y.max(0.0);
                shown >= rect.height * fraction
            })
        });
        let initial = if visible.get_untracked() {
            Seen::Seen
        } else {
            Seen::Waiting
        };
        let latch = machine(initial, || Latch, |_, ()| {});
        latch.feed(visible, |visible| visible);
        let state = latch.state();
        memo(move || state.get() == Seen::Seen)
    }
}
