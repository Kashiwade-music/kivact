//! Role: Registrations made on an owner while it is disposed (G-01).
//!
//! The public `Owner` cannot create signals or run setup, so the cases
//! that register on the reactive owner of a branch from inside its cleanup
//! live here, on a real-window `Screen`.

use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::Rc;
use std::time::Duration;

use crate::render;
use kivact_core::state::effect;
use kivact_core::task::spawn;
use kivact_core::view::{column, show, text};
use kivact_runtime::{Owner, on_cleanup, signal};

type Log = Rc<RefCell<Vec<String>>>;

fn push(log: &Log, entry: impl Into<String>) {
    log.borrow_mut().push(entry.into());
}

/// A cleanup that writes signals and registers more (a signal, an effect, a
/// task, a cleanup) on the owner being disposed leaves nothing behind and
/// reports nothing: the late cleanup runs at once, the rest is released with
/// the same disposal.
#[test]
fn a_cleanup_writes_signals_and_registers_on_its_disposing_owner() {
    let log: Log = Rc::default();
    // xtask-allow(state-fields): independent: a test slot holding the branch signals; written once
    let slots = Rc::new(Cell::new(None));
    let (seen, slot) = (log.clone(), slots.clone());
    let screen = render(move || {
        // xtask-allow(state-fields): independent: the show() condition the test flips to dispose the branch; the closed counter and the slot are written by other steps
        let open = signal(true);
        // xtask-allow(state-fields): independent: the counter a late cleanup writes after the branch is gone; the test reads it whether open is true or false
        let closed = signal(0);
        slot.set(Some((open, closed)));
        let seen = seen.clone();
        let branch = show(open, move || {
            let owner = Owner::current("disposal test", Location::caller());
            let late = seen.clone();
            on_cleanup(move || {
                // A write to a signal that outlives the branch.
                closed.update(|n| *n += 1);
                // A signal of the owner being disposed lives until the
                // cleanup returns.
                let own = owner.signal(1, Location::caller());
                own.set(2);
                push(&late, format!("own signal {}", own.get_untracked()));
                let (effect_log, task_log, cleanup_log) =
                    (late.clone(), late.clone(), late.clone());
                owner.with_setup(move || {
                    effect(move || push(&effect_log, "late effect ran"));
                    let _ = spawn(async move { push(&task_log, "late task ran") });
                    on_cleanup(move || push(&cleanup_log, "late setup cleanup"));
                });
            });
            text("child")
        });
        column((branch, text(move || format!("closed {}", closed.get()))))
    });
    let (open, closed) = slots.get().expect("setup ran");
    let residue = |screen: &crate::Screen| {
        let stats = screen.stats();
        (
            stats.live_reactive,
            stats.live_owners,
            stats.live_objects,
            stats.tree_nodes,
            stats.timers,
        )
    };
    open.set(false);
    screen.run_until_idle();
    let baseline = residue(&screen);
    screen.get_by_text("closed 1");
    screen.advance(Duration::from_millis(50));
    assert_eq!(
        *log.borrow(),
        vec!["own signal 2", "late setup cleanup"],
        "the late cleanup runs at once; the late effect and task never run"
    );
    assert_eq!(closed.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    // Open and close again: nothing piles up.
    for _ in 0..20 {
        open.set(true);
        screen.run_until_idle();
        open.set(false);
        screen.run_until_idle();
    }
    assert_eq!(
        residue(&screen),
        baseline,
        "registrations made while disposing leak"
    );
    screen.get_by_text("closed 21");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
