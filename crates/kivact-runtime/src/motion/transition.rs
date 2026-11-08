//! Role: Shared transition animation vocabulary.

use kivact_types::motion::{PlayDirection, Repeat};

pub(crate) use kivact_types::motion::Kind;
pub use kivact_types::motion::Transition;

/// Whether a run with `timing` settles back on its start: an alternating run
/// of an even, positive iteration count ends on `from` (the value of the end
/// of its last iteration, the rule of `Transition::step`).
pub(crate) fn settles_at_start(timing: &Transition) -> bool {
    matches!(timing.repeat, Repeat::Times(n) if n > 0 && n % 2 == 0)
        && timing.direction == PlayDirection::Alternate
}
