//! Role: Tests for edit phase handling in interactions.
//!
//! `EditPhase`: the Begin, Commit and Cancel a `splitter` handle reports around a drag,
//! a key press and an assistive technology action, so an app can make one undo
//! step of each.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::interaction::use_splitter;
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions};
use kivact::ui::{EditPhase, ScalarRange};
use kivact::view::A11yAction;
use kivact::view::MouseButton;

/// The phases an observer saw, each with the value in the binding at that time.
type Log = Rc<RefCell<Vec<(EditPhase, f64)>>>;

fn real_window() -> RenderOptions {
    RenderOptions {
        mirror: MirrorMode::Required,
        ..RenderOptions::from_env()
    }
}

fn range() -> ScalarRange {
    ScalarRange::new(0.0, 100.0).expect("invariant: finite range")
}

fn observer(value: Signal<f64>, log: &Log) -> impl Fn(EditPhase) + 'static {
    let log = log.clone();
    move |phase| log.borrow_mut().push((phase, value.get_untracked()))
}

fn phases(log: &Log) -> Vec<EditPhase> {
    log.borrow().iter().map(|(phase, _)| *phase).collect()
}

fn handle() -> Node {
    node().name("Split").style(style().size(8, 100))
}

use EditPhase::{Begin, Commit};

#[test]
fn a_drag_is_one_edit_from_press_to_release() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(40.0);
        let divider = handle().part(
            use_splitter(value, range(), Orientation::Horizontal)
                .on_edit(observer(value, &seen))
                .handle(),
        );
        (value, divider)
    });
    let rect = screen.get_by_role(Role::Splitter, "Split").rect().unwrap();
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 40.0)], "before the first write");
    screen.mouse_move(x + 10.0, y);
    screen.mouse_move(x + 25.0, y);
    assert_eq!(value.get_untracked(), 65.0);
    assert_eq!(
        phases(&log),
        [Begin],
        "the moves are not edits of their own"
    );
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        *log.borrow(),
        [(Begin, 40.0), (Commit, 65.0)],
        "after the last write"
    );

    // A press that never moves is still one edit, with nothing written.
    log.borrow_mut().clear();
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 65.0), (Commit, 65.0)]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn each_key_and_assistive_technology_write_is_its_own_edit() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(50.0);
        let divider = handle().part(
            use_splitter(value, range(), Orientation::Horizontal)
                .key_step(5.0)
                .unwrap()
                .reset_to(Some(20.0))
                .on_edit(observer(value, &seen))
                .handle(),
        );
        (value, divider)
    });
    let split = screen.get_by_role(Role::Splitter, "Split");
    split.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(*log.borrow(), [(Begin, 50.0), (Commit, 55.0)]);
    for (key, expected) in [
        (Key::ArrowLeft, 50.0),
        (Key::Home, 0.0),
        (Key::End, 100.0),
        (Key::Enter, 20.0),
    ] {
        let before = value.get_untracked();
        log.borrow_mut().clear();
        screen.press_key(key);
        assert_eq!(value.get_untracked(), expected, "{key:?}");
        assert_eq!(
            *log.borrow(),
            [(Begin, before), (Commit, expected)],
            "{key:?}"
        );
    }
    log.borrow_mut().clear();

    split.a11y(A11yAction::Increment);
    split.a11y(A11yAction::Decrement);
    split.a11y_set_value("73");
    assert_eq!(
        *log.borrow(),
        [
            (Begin, 20.0),
            (Commit, 25.0),
            (Begin, 25.0),
            (Commit, 20.0),
            (Begin, 20.0),
            (Commit, 73.0)
        ]
    );
    log.borrow_mut().clear();

    // Input that writes nothing is no edit.
    split.a11y_set_value("not a number");
    split.a11y_set_value("NaN");
    screen.press_key(Key::ArrowUp);
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
