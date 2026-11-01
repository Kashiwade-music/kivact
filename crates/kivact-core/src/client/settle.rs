//! Role: Run the tasks and timers of a turn until quiet.

use kivact_reactive::{ErrorKind, Runtime, TurnRecorder, UiError};
use kivact_task::TaskState;

/// Rounds of tasks and timers one settle runs; the rest waits for the next
/// turn.
const SETTLE_ROUNDS: usize = 1000;

/// Run tasks and timers until quiet; stabilise after each round. Work left
/// after [`SETTLE_ROUNDS`] is reported (a `Cycle` to the default sink) and
/// runs in the next turn.
pub(super) fn settle_async(rt: &Runtime, turn: &TurnRecorder<'_>) {
    for _ in 0..SETTLE_ROUNDS {
        let wakers = TaskState::of(rt).take_due_timers(rt.now_ns());
        let woke = !wakers.is_empty();
        if woke {
            turn.cause("timer".into());
        }
        for w in wakers {
            w.wake();
        }
        let n = TaskState::of(rt).poll_tasks(rt);
        if n > 0 {
            turn.cause("task".into());
        }
        rt.stabilize();
        if n == 0 && !woke {
            return;
        }
    }
    let tasks = TaskState::of(rt);
    if tasks.has_ready() || tasks.next_deadline().is_some_and(|t| t <= rt.now_ns()) {
        rt.report(
            None,
            UiError::new(
                ErrorKind::Cycle { path: Vec::new() },
                format!(
                    "tasks and timers did not settle within {SETTLE_ROUNDS} rounds of one turn \
                 (a task that wakes itself, a timer due again at once); the rest runs in the \
                 next turn"
                ),
                None,
            ),
        );
    }
}
