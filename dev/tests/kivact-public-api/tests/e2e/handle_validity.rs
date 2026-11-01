//! Role: Tests for handle and reference validity checking.
//!
//! Handle validity: handles and node refs of another App, or of an owner
//! that was disposed, are rejected (`WrongApp`, `Disposed`) with a readable
//! message or a structured error, and writes to them are observable as
//! diagnostics instead of silently applying.

use kivact::view::callback;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::Callback;
use kivact::diagnostics::ErrorKind;
use kivact::prelude::*;
use kivact::test::{Screen, render};

fn kinds(screen: &Screen) -> Vec<ErrorKind> {
    screen.errors().iter().map(|e| e.kind().clone()).collect()
}

#[test]
fn outside_a_frame_does_not_guess_a_live_app() {
    use kivact::primitive::{Owner, clock_now, focused_node};
    use std::time::Duration;

    let (screen_a, a) = mount(|| (signal(1), text("App A")));
    let (screen_b, b) = mount(|| {
        (
            signal(2),
            node()
                .role(Role::Button)
                .test_id("App B focus")
                .focusable(true)
                .style(style().size(40, 40)),
        )
    });
    screen_b.get_by_test_id("App B focus").focus();
    screen_b.advance(Duration::from_millis(10));
    assert!(
        focused_node().is_none(),
        "outside a frame must not select the newest App"
    );
    assert_eq!(clock_now(), 0, "no App clock applies outside a frame");

    let root_at = line!() + 1;
    let root = std::panic::catch_unwind(|| Owner::root("outside_root"));
    let duration = Duration::from_secs(1);
    let sleep_at = line!() + 1;
    let sleep = std::panic::catch_unwind(|| drop(kivact::task::sleep(duration)));
    let timeout_at = line!() + 1;
    let timeout = std::panic::catch_unwind(|| drop(kivact::task::timeout(duration, async {})));
    let setup_at = line!() + 2;
    let setup = std::panic::catch_unwind(|| {
        let _ = signal(3);
    });
    for (result, kind, at) in [
        (root.map(|_| ()), "WrongApp", root_at),
        (sleep, "WrongApp", sleep_at),
        (timeout, "WrongApp", timeout_at),
        (setup, "SetupOnly", setup_at),
    ] {
        let payload = result.expect_err("an App must not be inferred outside a frame");
        let message = payload.downcast_ref::<String>().expect("readable misuse");
        assert!(message.contains(kind), "{message}");
        assert!(message.contains(&format!("{}:{at}", file!())), "{message}");
    }
    // Explicit handles still select their own Apps without an executing frame.
    a.set(3);
    b.set(4);
    screen_a.run_until_idle();
    screen_b.run_until_idle();
    assert_eq!(a.get_untracked(), 3);
    assert_eq!(b.get_untracked(), 4);
    assert!(screen_a.errors().is_empty());
    assert!(screen_b.errors().is_empty());
}

#[test]
fn disposed_handles_panic_on_get_none_on_try_get_and_ignore_writes() {
    let leaked: Rc<Cell<Option<Signal<i32>>>> = Rc::new(Cell::new(None));
    let l2 = leaked.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        (
            open,
            show(open, move || {
                let inner = signal(7);
                l2.set(Some(inner));
                text(move || inner.get().to_string())
            }),
        )
    });
    let inner = leaked.get().expect("child mounted");
    assert_eq!(inner.try_get(), Some(7));
    open.set(false);
    screen.run_until_idle();
    assert_eq!(inner.try_get(), None);
    inner.set(1); // ignored
    let r = std::panic::catch_unwind(|| inner.get());
    let err = r.expect_err("get after dispose panics");
    let _ = err;
    // remount: a new signal may reuse the slot but the old handle stays dead
    open.set(true);
    screen.run_until_idle();
    assert_eq!(inner.try_get(), None);
    assert_eq!(leaked.get().and_then(|s| s.try_get()), Some(7));
}

#[test]
fn handles_of_a_dropped_app_are_wrong_app() {
    let (screen, s) = mount(|| {
        let s = signal(3);
        (s, text(move || s.get().to_string()))
    });
    let (screen2, s2) = mount(|| {
        let s = signal(4);
        (s, text(move || s.get().to_string()))
    });
    assert_eq!(s.get_untracked(), 3);
    drop(screen);
    assert_eq!(s.try_get(), None);
    assert_eq!(s2.get_untracked(), 4);
    let r = std::panic::catch_unwind(|| s.get_untracked());
    assert!(r.is_err());
    drop(screen2);
}

// ---------------------------------------------------------------------------
// Regression (design survey): a NodeRef of another App is bound in this App (wrong target).

type Refs = Rc<RefCell<Vec<NodeRef>>>;

fn two_refs_app(refs: Refs, foreign: Option<NodeRef>) -> Screen {
    render(move || {
        let own = node_ref();
        refs.borrow_mut().push(own);
        let b = node()
            .role(Role::Group)
            .name("target")
            .focusable(true)
            .style(style().size(10, 10));
        match foreign {
            Some(f) => b.node_ref(f),
            None => b.node_ref(own),
        }
    })
}

#[test]
fn node_ref_of_another_app_does_not_bind_to_this_apps_ref() {
    let refs_a: Refs = Rc::new(RefCell::new(Vec::new()));
    let screen_a = two_refs_app(refs_a.clone(), None);
    let ra = refs_a.borrow()[0];
    // App B creates its own (unbound) NodeRef first, then binds App A's ref.
    let refs_b: Refs = Rc::new(RefCell::new(Vec::new()));
    let screen_b = two_refs_app(refs_b.clone(), Some(ra));
    let rb = refs_b.borrow()[0];
    // rb was never passed to `.node_ref(..)`: it must not point at App B's box.
    assert!(
        kivact::diagnostics::why(&rb).is_none(),
        "App B's unbound NodeRef got bound to the box that received App A's ref"
    );
    let errs = kinds(&screen_b);
    assert!(
        errs.contains(&ErrorKind::WrongApp),
        "binding a foreign NodeRef must be reported as WrongApp, got {errs:?}"
    );
    drop(screen_a);
}

// ---------------------------------------------------------------------------
// Regression (design survey): handles of another (live) App are silently usable: reads are not
// tracked and writes bypass the purity check.

#[test]
fn another_apps_signal_is_rejected_inside_a_computation() {
    let (screen_a, sig_a) = mount(|| {
        let s = signal(1);
        (s, text(move || s.get().to_string()))
    });
    let (screen_b, ()) = mount(move || ((), text(move || format!("b sees {}", sig_a.get()))));
    sig_a.set(2);
    screen_a.run_until_idle();
    screen_b.run_until_idle();
    let shown = screen_b.query_by_text("b sees 2").is_some();
    let rejected = kinds(&screen_b).contains(&ErrorKind::WrongApp);
    assert!(
        shown || rejected,
        "cross-App read neither tracked nor rejected: {}",
        screen_b.debug()
    );
}

#[test]
fn another_apps_signal_cannot_be_written_from_a_pure_context() {
    let (screen_a, sig_a) = mount(|| {
        let s = signal(1);
        (s, text(move || s.get().to_string()))
    });
    let (screen_b, ()) = mount(move || {
        let m = memo(move || {
            sig_a.set(99);
            0
        });
        ((), text(move || m.get().to_string()))
    });
    screen_a.run_until_idle();
    assert_eq!(
        sig_a.get_untracked(),
        1,
        "memo of App B wrote App A's signal"
    );
    let _ = screen_b;
}

// ---------------------------------------------------------------------------
// Regression (design survey): a misuse panic that is not caught by a boundary has no message.

#[test]
fn misuse_outside_a_boundary_panics_with_a_message() {
    let leaked: Rc<Cell<Option<Signal<i32>>>> = Rc::new(Cell::new(None));
    let l2 = leaked.clone();
    let screen = render(move || l2.set(Some(signal(1))));
    drop(screen);
    let s = leaked.get().expect("signal");
    let r = std::panic::catch_unwind(|| s.get());
    let p = r.expect_err("WrongApp panic");
    // The payload is an opaque MisusePanic: neither &str nor String, and the
    // quiet panic hook prints nothing, so the test log shows no reason at all.
    let readable = p.downcast_ref::<String>().is_some() || p.downcast_ref::<&str>().is_some();
    assert!(
        readable,
        "misuse panic outside a boundary carries no readable message"
    );
}

// ---------------------------------------------------------------------------
// Regression (design survey): `try_call` on a disposed callback panics instead of returning Err.

#[test]
fn try_call_on_a_disposed_callback_returns_err() {
    let leaked: Rc<Cell<Option<Callback<u32>>>> = Rc::new(Cell::new(None));
    let l2 = leaked.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let v = show(open, move || {
            l2.set(Some(callback(|_| {})));
            text("child")
        });
        (open, v)
    });
    open.set(false);
    screen.run_until_idle();
    let cb = leaked.get().expect("callback");
    let r = std::panic::catch_unwind(|| cb.try_call(1));
    assert!(
        matches!(r, Ok(Err(_))),
        "try_call on a disposed callback must return Err, not panic"
    );
}

// ---------------------------------------------------------------------------
// Regression (design survey): WriteAfterDispose is recorded where no public API can read it.

#[test]
fn write_after_dispose_diagnostic_is_observable() {
    let leaked: Rc<Cell<Option<Signal<i32>>>> = Rc::new(Cell::new(None));
    let l2 = leaked.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let v = show(open, move || {
            l2.set(Some(signal(1)));
            text("child")
        });
        (open, v)
    });
    open.set(false);
    screen.run_until_idle();
    leaked.get().expect("signal").set(5);
    screen.run_until_idle();
    let in_errors = kinds(&screen).contains(&ErrorKind::WriteAfterDispose);
    let in_trace = screen
        .trace()
        .iter()
        .any(|t| t.notes.iter().any(|n| n.contains("dispose")));
    assert!(
        in_errors || in_trace,
        "the WriteAfterDispose diagnostic only reaches tracing::debug and a private ring"
    );
}

// ---------------------------------------------------------------------------
// Regression (design survey): a disposed setup context still allocates during another setup
// (survey S14). No handle names an owner to create in; the code
// of a disposed branch can only be reached through a handle it created, and
// that handle is rejected.

#[test]
fn disposed_branch_code_is_rejected_during_another_setup() {
    let stashed: Rc<Cell<Option<Callback<()>>>> = Rc::new(Cell::new(None));
    let st = stashed.clone();
    let made = Rc::new(Cell::new(false));
    let m2 = made.clone();
    let (screen, (a, b)) = mount(move || {
        let a = signal(true);
        let b = signal(false);
        let st2 = st.clone();
        let v = column((
            show(a, move || {
                let m3 = m2.clone();
                st2.set(Some(callback(move |()| {
                    signal(1);
                    m3.set(true);
                })));
                text("a")
            }),
            show(b, move || {
                if let Some(old) = st.get() {
                    old.call(()); // created by the old, disposed branch
                }
                text("b")
            }),
        ));
        ((a, b), v)
    });
    a.set(false);
    screen.run_until_idle();
    b.set(true);
    screen.run_until_idle();
    assert!(
        kinds(&screen)
            .iter()
            .any(|k| matches!(k, ErrorKind::Disposed { .. })),
        "signal created through a disposed branch (completed: {})",
        made.get()
    );
    assert!(!made.get());
}
