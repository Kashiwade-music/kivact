//! Role: Tests for purity requirements in contexts and effects.
//!
//! Pure contexts: memos, render effects and the closures that flows, style
//! and resources evaluate reject writes and host side effects
//! (`WriteInPureContext`), even under `untrack`, and the rejected write
//! changes nothing.

use crate::support::mount;
use kivact::diagnostics::ErrorKind;
use kivact::motion::{Easing, Transition};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::test::Screen;

fn kinds(screen: &Screen) -> Vec<ErrorKind> {
    screen.errors().iter().map(|e| e.kind().clone()).collect()
}

#[test]
fn writes_in_pure_context_are_rejected_even_under_untrack() {
    let (screen, (a, victim)) = mount(move || {
        let a = signal(1);
        let victim = signal(0);
        let m = memo(move || {
            let v = a.get();
            if v > 1 {
                kivact::untrack(|| victim.set(99)); // purity is inherited
            }
            v * 10
        });
        ((a, victim), text(move || m.get().to_string()))
    });
    screen.get_by_text("10");
    a.set(2);
    screen.run_until_idle();
    assert_eq!(victim.get_untracked(), 0);
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
    // the memo keeps its previous value
    screen.get_by_text("10");
}

#[test]
fn render_effects_are_pure() {
    let (screen, (a, victim)) = mount(move || {
        let a = signal(0);
        let victim = signal(0);
        (
            (a, victim),
            text(move || {
                if a.get() == 1 {
                    victim.set(1);
                }
                a.get().to_string()
            }),
        )
    });
    a.set(1);
    screen.run_until_idle();
    assert_eq!(victim.get_untracked(), 0);
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
}

// ---------------------------------------------------------------------------
// Regression (design survey): user closures evaluated by Internal effects are not pure.

fn pure_violation_in<V: IntoView>(
    build: impl FnOnce(Signal<i32>, Signal<i32>) -> V + 'static,
) -> (Screen, Signal<i32>, Signal<i32>) {
    let (screen, (trigger, victim)) = mount(move || {
        let trigger = signal(0);
        let victim = signal(0);
        ((trigger, victim), build(trigger, victim))
    });
    trigger.set(1);
    screen.run_until_idle();
    (screen, trigger, victim)
}

#[test]
fn show_condition_closure_is_pure() {
    let (screen, _, victim) = pure_violation_in(|trigger, victim| {
        show(
            move || {
                if trigger.get() == 1 {
                    victim.set(42);
                }
                true
            },
            || text("x"),
        )
    });
    assert_eq!(victim.get_untracked(), 0);
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
}

#[test]
fn for_each_items_and_key_closures_are_pure() {
    let (screen, _, victim) = pure_violation_in(|trigger, victim| {
        for_each(
            move || {
                if trigger.get() == 1 {
                    victim.set(42);
                }
                vec![1, 2, 3]
            },
            |v| *v,
            |k, _| text(k.to_string()),
        )
    });
    assert_eq!(victim.get_untracked(), 0);
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
}

#[test]
fn style_closure_is_pure_even_with_a_paint_transition() {
    let (screen, _, victim) = pure_violation_in(|trigger, victim| {
        node().style(move || {
            if trigger.get() == 1 {
                victim.set(42);
            }
            style().opacity(1.0).transition(
                TransitionProp::Paint,
                Transition::tween(100, Easing::Linear),
            )
        })
    });
    assert_eq!(victim.get_untracked(), 0);
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
}

#[test]
fn resource_source_closure_is_pure() {
    let (screen, _, victim) = pure_violation_in(|trigger, victim| {
        let r = resource(
            move || {
                if trigger.get() == 1 {
                    victim.set(42);
                }
                trigger.get()
            },
            |v: i32| async move { Ok(v) },
        );
        text(move || r.latest().unwrap_or(0).to_string())
    });
    assert_eq!(victim.get_untracked(), 0);
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
}

// ---------------------------------------------------------------------------
// Regression (design survey): side-effecting methods are accepted inside memos (inconsistent with
// spawn / Action::run, which are rejected).

#[test]
fn host_side_effects_are_rejected_in_a_memo() {
    let (screen, arm) = mount(|| {
        let arm = signal(false);
        let r = node_ref();
        let m = memo(move || {
            if arm.get() {
                r.focus();
                drop(platform::copy_text("from a memo"));
            }
            0
        });
        (
            arm,
            column((
                text(move || m.get().to_string()),
                node()
                    .focusable(true)
                    .role(Role::Group)
                    .name("t")
                    .node_ref(r),
            )),
        )
    });
    arm.set(true);
    screen.run_until_idle();
    assert!(
        screen.focused().is_none(),
        "a memo moved the keyboard focus"
    );
    assert_eq!(screen.clipboard(), None, "a memo wrote the OS clipboard");
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
}
