//! Role: Key presses nobody handles are reported; used ones are not.

use crate::support::mount;
use kivact::prelude::*;
use kivact::view::{Handled, Key, KeyCode, Modifiers};

#[test]
fn unhandled_key_presses_are_reported_once_and_releases_are_not() {
    let (screen, ()) = mount(|| ((), node().style(style().size(200, 100))));
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(screen.unhandled_keys(), [KeyCode::Space]);
    assert!(
        screen.unhandled_keys().is_empty(),
        "a release is not a press and the report was read"
    );
    screen.key_down(Key::ArrowRight, KeyCode::ArrowRight, Modifiers::default());
    screen.key_up(Key::ArrowRight, KeyCode::ArrowRight, Modifiers::default());
    assert_eq!(screen.unhandled_keys(), [KeyCode::ArrowRight]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn handled_presses_and_activation_are_not_reported() {
    let (screen, pressed) = mount(|| {
        let pressed = signal(0);
        let count = pressed;
        let button = node()
            .role(Role::Button)
            .focusable(true)
            .test_id("ok")
            .on_key(|_| Handled::Yes)
            .on_press(move || count.set(count.get_untracked() + 1))
            .style(style().size(100, 40));
        (
            pressed,
            column((
                button,
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .test_id("enter")
                    .on_press(move || pressed.set(pressed.get_untracked() + 1))
                    .style(style().size(100, 40)),
            )),
        )
    });
    screen.get_by_test_id("ok").focus();
    screen.key_down(Key::Char('x'), KeyCode::KeyX, Modifiers::default());
    screen.key_up(Key::Char('x'), KeyCode::KeyX, Modifiers::default());
    assert!(
        screen.unhandled_keys().is_empty(),
        "an on_key handler took the press"
    );
    screen.get_by_test_id("enter").focus();
    screen.key_down(Key::Enter, KeyCode::Enter, Modifiers::default());
    screen.key_up(Key::Enter, KeyCode::Enter, Modifiers::default());
    assert!(
        screen.unhandled_keys().is_empty(),
        "Enter activated the focused control"
    );
    assert_eq!(pressed.get_untracked(), 1, "the default action ran");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
