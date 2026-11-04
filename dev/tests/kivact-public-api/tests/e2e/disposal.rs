//! Role: Tests for registrations made while an owner is disposed (G-01).
//! A cleanup that writes signals or registers a cleanup on the owner being
//! disposed leaves nothing behind and reports nothing: the runtime runs a
//! late cleanup at once. (Registering signals, effects and tasks on a
//! disposing owner needs the reactive owner and is tested in
//! `kivact-core`'s `disposal_tests`.) A component whose handlers hold values
//! whose `Drop` uses Kivact goes away with a branch switch.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use crate::support::random::residue;
use kivact::custom::Owner;
use kivact::geometry::Point;
use kivact::prelude::*;

type Log = Rc<RefCell<Vec<String>>>;

fn log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

fn push(log: &Log, entry: impl Into<String>) {
    log.borrow_mut().push(entry.into());
}

#[test]
fn a_cleanup_writes_signals_and_registers_a_cleanup_on_its_disposing_owner() {
    let l = log();
    let l2 = l.clone();
    let (screen, (open, closed)) = mount(move || {
        let open = signal(true);
        let closed = signal(0);
        let branch = show(open, move || {
            let owner = Owner::current();
            let l3 = l2.clone();
            on_cleanup(move || {
                // A write to a signal that outlives the branch.
                closed.update(|n| *n += 1);
                push(&l3, "cleanup ran");
                let late = l3.clone();
                owner.on_cleanup(move || push(&late, "late cleanup"));
            });
            text("child")
        });
        (
            (open, closed),
            column((branch, text(move || format!("closed {}", closed.get())))),
        )
    });
    open.set(false);
    screen.run_until_idle();
    let baseline = residue(&screen);
    screen.get_by_text("closed 1");
    screen.advance(std::time::Duration::from_millis(50));
    assert_eq!(
        *l.borrow(),
        vec!["cleanup ran", "late cleanup"],
        "the late cleanup runs at once"
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

/// Reads the tree of the App when dropped (`NodeRef::hit_test`).
struct ReadsTreeOnDrop {
    target: NodeRef,
    dropped: Rc<Cell<bool>>,
}

impl Drop for ReadsTreeOnDrop {
    fn drop(&mut self) {
        let _ = self.target.hit_test(Point { x: 1.0, y: 1.0 });
        self.dropped.set(true);
    }
}

#[test]
fn a_handler_value_whose_drop_uses_kivact_goes_away_with_its_branch() {
    let dropped = Rc::new(Cell::new(false));
    let d2 = dropped.clone();
    let (screen, (open, presses)) = mount(move || {
        let open = signal(true);
        let presses = signal(0);
        let target = node_ref();
        let branch = show(open, move || {
            let held = ReadsTreeOnDrop {
                target,
                dropped: d2.clone(),
            };
            let label = ("inner").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
                .on_press(move || {
                    let _ = &held;
                    presses.update(|n| *n += 1);
                })
        });
        (
            (open, presses),
            column((branch, text("outer")))
                .node_ref(target)
                .style(style().size(200, 120)),
        )
    });
    screen.get_by_role(Role::Button, "inner").click();
    assert_eq!(presses.get_untracked(), 1);
    assert!(!dropped.get());
    open.set(false);
    screen.run_until_idle();
    assert!(
        dropped.get(),
        "the handler's value was dropped with its node"
    );
    assert!(screen.query_by_role(Role::Button, "inner").is_none());
    screen.get_by_text("outer");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
