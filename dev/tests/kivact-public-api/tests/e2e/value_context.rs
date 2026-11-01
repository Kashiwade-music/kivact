//! Role: Constant lifetime and dependency collectors through public real windows.
use kivact::diagnostics::ErrorKind;
use kivact::prelude::*;
use kivact::test::{RenderOptions, WindowMode, render_with, render_with_handle};
use kivact::view::{callback, constant, selector};
use std::cell::Cell;
use std::rc::Rc;

struct Payload(Rc<Cell<usize>>);
type SavedConstant = Rc<Cell<Option<(Value<Payload>, u32)>>>;
impl Drop for Payload {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn constants_are_copy_without_clone_and_keep_caller_lifetime_and_creation_site() {
    let saved: SavedConstant = Rc::new(Cell::new(None));
    let drops = Rc::new(Cell::new(0));
    let (slot, dropped) = (saved.clone(), drops.clone());
    let (screen, (open, read)) = render_with_handle(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        move || {
            let open = signal(true);
            let created = dropped.clone();
            let store = slot.clone();
            let make = callback(move |()| {
                let at = line!() + 1;
                let value = constant(Payload(created.clone()));
                let copy = value;
                value.with(|_| ());
                copy.with(|_| ());
                store.set(Some((value, at)));
            });
            let lookup = slot.clone();
            let read = callback(move |()| lookup.get().expect("made constant").0.with(|_| ()));
            (
                (open, read),
                show(open, move || {
                    make.call(());
                    text("child")
                }),
            )
        },
    );
    read.try_call(()).unwrap();
    let (old, at) = saved.get().unwrap();
    open.set(false);
    screen.run_until_idle();
    assert_eq!(drops.get(), 1);
    let error = read
        .try_call(())
        .expect_err("caller branch disposed its constant");
    match error.kind() {
        ErrorKind::Disposed {
            created_at: Some(created),
            disposed_by: Some(_),
        } => assert_eq!(created.line(), at),
        other => panic!("unexpected constant lifetime diagnostic: {other:?}"),
    }
    open.set(true);
    screen.run_until_idle();
    let stale = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| old.with(|_| ())));
    assert!(stale.is_err());
    assert_eq!(drops.get(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn constants_reject_foreign_apps_and_dropped_apps() {
    let (first, value) = render_with_handle(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        || (constant(String::from("first")), text("first App")),
    );
    let second = render_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        move || text(move || value.get()),
    );
    assert!(
        second
            .errors()
            .iter()
            .any(|e| matches!(e.kind(), ErrorKind::WrongApp))
    );
    assert_eq!(value.get_untracked(), "first");
    drop(first);
    assert_eq!(value.try_get(), None);
    assert!(std::panic::catch_unwind(|| value.get()).is_err());
}

#[test]
fn selector_pruning_keeps_the_suspended_outer_collectors_dependencies() {
    let (screen, selected) = render_with_handle(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        || {
            let selected = signal(Some(0usize));
            let selection = selector(selected);
            let label = memo(move || {
                let zero = selection.is(&0);
                kivact::untrack(|| {
                    for key in 1..2000 {
                        let _ = selection.is(&key);
                    }
                });
                if zero { "zero" } else { "other" }
            });
            (selected, text(move || label.get()))
        },
    );
    screen.get_by_text("zero");
    selected.set(Some(1));
    screen.run_until_idle();
    screen.get_by_text("other");
    selected.set(Some(0));
    screen.run_until_idle();
    screen.get_by_text("zero");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
