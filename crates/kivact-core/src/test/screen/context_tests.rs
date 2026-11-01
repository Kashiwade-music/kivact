//! Role: Real-window regressions for layer context entry and unwind restoration.
use crate::prelude::{Value, memo, signal, text};
use crate::test::{RenderOptions, WindowMode, render_with_handle};
use kivact_reactive::{ErrorKind, Owner, UiError, pure, untrack};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, Location, catch_unwind};

#[test]
fn pure_setup_rejects_writes_and_unwind_restores_owner_purity_and_tracking() {
    let (screen, (violation, restored, value, a, b, c)) = render_with_handle(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::default()
        },
        || {
            let owner = Owner::current("context test", Location::caller());
            let value = signal(0);
            let violation = catch_unwind(AssertUnwindSafe(|| {
                pure(|| untrack(|| value.set(99)));
            }))
            .err()
            .map(|p| UiError::from_panic(p, None));
            let restored = Owner::current("context test", Location::caller()) == owner
                && Owner::ambient("context test", Location::caller()) == owner;
            let _created_after_unwind = signal(1);
            value.set(7);
            let (a, b, c) = (signal(1), signal(10), signal(100));
            let sum = memo(move || {
                let first = a.get();
                let middle = Cell::new(0);
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    pure(|| {
                        middle.set(b.get());
                        panic!("test unwind in a copied pure context");
                    })
                }));
                first + middle.get() + c.get()
            });
            (
                (violation, restored, value, a, b, c),
                text(move || format!("Sum {}", sum.get())),
            )
        },
    );
    assert_eq!(
        violation.as_ref().map(UiError::kind),
        Some(&ErrorKind::WriteInPureContext),
        "pure must reject a setup-time write, including through untrack"
    );
    assert!(restored);
    assert_eq!(value.get_untracked(), 7);
    screen.get_by_text("Sum 111");
    b.set(20);
    screen.run_until_idle();
    screen.get_by_text("Sum 121");
    c.set(200);
    screen.run_until_idle();
    screen.get_by_text("Sum 221");
    a.set(2);
    screen.run_until_idle();
    screen.get_by_text("Sum 222");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn copied_pure_and_untracked_contexts_do_not_duplicate_cycle_path() {
    let screen = crate::test::render_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::default()
        },
        || {
            let slot = std::rc::Rc::new(Cell::new(None::<Value<i32>>));
            let read = slot.clone();
            let first = memo(move || pure(|| untrack(|| read.get().unwrap().get())));
            let second = memo(move || first.get());
            slot.set(Some(second));
            text(move || second.get().to_string())
        },
    );
    let errors = screen.errors();
    let path = errors
        .iter()
        .find_map(|error| match error.kind() {
            ErrorKind::Cycle { path } => Some(path),
            _ => None,
        })
        .expect("memo cycle is reported");
    assert!(path.len() >= 2, "{path:?}");
    let unique: std::collections::HashSet<_> = path
        .iter()
        .map(|at| (at.file(), at.line(), at.column()))
        .collect();
    assert_eq!(
        path.len(),
        unique.len(),
        "copied contexts are not computations: {path:?}"
    );
}
