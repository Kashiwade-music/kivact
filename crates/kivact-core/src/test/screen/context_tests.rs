//! Role: Real-window regressions for layer context entry.
use crate::prelude::{Value, memo, signal, text};
use crate::test::{RenderOptions, WindowMode, render_with, render_with_handle};
use kivact_reactive::{pure, untrack};
use std::cell::Cell;

#[test]
#[should_panic(expected = "WriteInPureContext")]
fn pure_setup_rejects_writes_even_through_untrack() {
    render_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::default()
        },
        || {
            let value = signal(0);
            pure(|| untrack(|| value.set(99)));
            text("unreachable")
        },
    );
}

#[test]
fn pure_section_inside_a_memo_keeps_tracking() {
    let (screen, (a, b, c)) = render_with_handle(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::default()
        },
        || {
            let (a, b, c) = (signal(1), signal(10), signal(100));
            let sum = memo(move || a.get() + pure(|| b.get()) + c.get());
            ((a, b, c), text(move || format!("Sum {}", sum.get())))
        },
    );
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
#[should_panic(expected = "memo reads itself (dependency cycle)")]
fn a_memo_cycle_through_copied_contexts_panics() {
    render_with(
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
}
