//! Role: Tests for `Binding::update` on both kinds of binding.
//!
//! A binding of a `Signal` changes the signal in place (and notifies even
//! when the value is equal); a `binding(get, set)` reads the value, applies
//! the closure and calls `set` once. Both are driven here from a button's
//! `on_press`, and the switch and its accessibility state must follow.

use crate::support::mount;
use kivact::prelude::*;

#[test]
fn updating_a_signal_binding_flips_the_switch_and_its_accessibility_state() {
    let (screen, checked) = mount(|| {
        let checked = signal(false);
        let view = column((
            kivact::ui::switch("Airplane mode", checked),
            button("Toggle").on_press(move || checked.update(|value| *value = !*value)),
        ));
        (checked, view)
    });
    assert_eq!(
        screen.get_by_role(Role::Switch, "Airplane mode").checked(),
        Some(CheckedState::Unchecked)
    );
    screen.get_by_role(Role::Button, "Toggle").click();
    assert_eq!(
        screen.get_by_role(Role::Switch, "Airplane mode").checked(),
        Some(CheckedState::Checked)
    );
    assert!(checked.get_untracked());
    screen.get_by_role(Role::Button, "Toggle").click();
    assert_eq!(
        screen.get_by_role(Role::Switch, "Airplane mode").checked(),
        Some(CheckedState::Unchecked)
    );
    assert!(!checked.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn updating_a_getter_setter_binding_calls_the_setter_once() {
    let (screen, (value, calls)) = mount(|| {
        let value = signal(false);
        let calls = var(0u32);
        let bound = binding(
            move || value.get(),
            move |next| {
                calls.update(|count| *count += 1);
                value.set(next);
            },
        );
        let view = column((
            kivact::ui::switch("Night mode", bound),
            button("Toggle").on_press(move || bound.update(|value| *value = !*value)),
        ));
        ((value, calls), view)
    });
    screen.get_by_role(Role::Button, "Toggle").click();
    assert_eq!(calls.get(), 1, "one update, one setter call");
    assert!(value.get_untracked());
    assert_eq!(
        screen.get_by_role(Role::Switch, "Night mode").checked(),
        Some(CheckedState::Checked)
    );
    screen.get_by_role(Role::Button, "Toggle").click();
    assert_eq!(calls.get(), 2);
    assert!(!value.get_untracked());
    assert_eq!(
        screen.get_by_role(Role::Switch, "Night mode").checked(),
        Some(CheckedState::Unchecked)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
