//! Role: Tests for edit phase handling in interactions.
//!
//! `EditPhase`: the Begin, Commit and Cancel a `splitter` handle reports around a drag,
//! a key press and an assistive technology action, so an app can make one undo
//! step of each.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::interaction::{EditSession, use_splitter};
use kivact::prelude::*;
use kivact::test::{RenderOptions, WindowMode};
use kivact::ui::{EditPhase, ScalarRange};
use kivact::view::A11yAction;
use kivact::view::{MouseButton, PointerKind};

/// The phases an observer saw, each with the value in the binding at that time.
type Log = Rc<RefCell<Vec<(EditPhase, f64)>>>;

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
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

use EditPhase::{Begin, Cancel, Commit};

#[test]
fn once_reports_a_whole_edit_and_joins_an_active_edit() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, (value, edit)) = mount_with(real_window(), move || {
        let value = signal(10.0);
        let edit =
            EditSession::new(move |initial| value.set(initial)).on_edit(observer(value, &seen));
        ((value, edit), text(move || format!("{}", value.get())))
    });
    edit.once(10.0, || value.set(20.0));
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 20.0)]);
    assert!(!edit.is_active());
    log.borrow_mut().clear();
    edit.begin(20.0);
    edit.once(999.0, || value.set(30.0));
    assert_eq!(*log.borrow(), [(Begin, 20.0)]);
    assert!(edit.is_active());
    assert!(edit.cancel());
    assert_eq!(*log.borrow(), [(Begin, 20.0), (Cancel, 20.0)]);
    assert_eq!(value.get_untracked(), 20.0);
    screen.run_until_idle();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

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
fn escape_during_a_drag_restores_the_start_and_cancels_once() {
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
    let split = screen.get_by_role(Role::Splitter, "Split");
    let rect = split.rect().unwrap();
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 30.0, y);
    assert_eq!(value.get_untracked(), 70.0);
    screen.press_key(Key::Escape);
    assert_eq!(
        *log.borrow(),
        [(Begin, 40.0), (Cancel, 40.0)],
        "the start is back before Cancel"
    );
    assert_eq!(value.get_untracked(), 40.0);
    screen.mouse_move(x + 50.0, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        phases(&log),
        [Begin, Cancel],
        "the rest of the pointer gesture reports nothing"
    );
    screen.press_key(Key::Escape);
    assert_eq!(phases(&log), [Begin, Cancel], "no edit, no Cancel");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_engine_canceling_the_pointer_restores_the_start_and_cancels() {
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
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_move(contact, x + 20.0, y);
    assert_eq!(value.get_untracked(), 60.0);
    screen.contact_cancel(contact);
    assert_eq!(*log.borrow(), [(Begin, 40.0), (Cancel, 40.0)]);
    assert_eq!(value.get_untracked(), 40.0);
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
    split.a11y_with_value(A11yAction::SetValue, "73");
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
    split.a11y_with_value(A11yAction::SetValue, "not a number");
    split.a11y_with_value(A11yAction::SetValue, "NaN");
    screen.press_key(Key::ArrowUp);
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_double_click_reset_is_one_edit() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, _) = mount_with(real_window(), move || {
        let value = signal(80.0);
        let divider = handle().part(
            use_splitter(value, range(), Orientation::Horizontal)
                .reset_to(Some(25.0))
                .on_edit(observer(value, &seen))
                .handle(),
        );
        (value, divider)
    });
    screen.get_by_role(Role::Splitter, "Split").double_click();
    assert_eq!(
        *log.borrow(),
        [(Begin, 80.0), (Commit, 80.0), (Begin, 80.0), (Commit, 25.0)],
        "the first click is a press that wrote nothing; the second press resets"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disposing_the_splitter_during_a_drag_cancels_it() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, (value, present)) = mount_with(real_window(), move || {
        let value = signal(40.0);
        let present = signal(true);
        let view = show(present, move || {
            handle().part(
                use_splitter(value, range(), Orientation::Horizontal)
                    .on_edit(observer(value, &seen))
                    .handle(),
            )
        });
        ((value, present), view)
    });
    let rect = screen.get_by_role(Role::Splitter, "Split").rect().unwrap();
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 20.0, y);
    assert_eq!(value.get_untracked(), 60.0);
    present.set(false);
    screen.run_until_idle();
    assert_eq!(
        *log.borrow(),
        [(Begin, 40.0), (Cancel, 40.0)],
        "the start value is back before Cancel"
    );
    assert_eq!(value.get_untracked(), 40.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(phases(&log), [Begin, Cancel]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disabling_the_splitter_during_a_drag_ends_it_where_it_is() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, (value, disabled)) = mount_with(real_window(), move || {
        let value = signal(40.0);
        let disabled = signal(false);
        let divider = handle().part(
            use_splitter(value, range(), Orientation::Horizontal)
                .disabled(disabled)
                .on_edit(observer(value, &seen))
                .handle(),
        );
        ((value, disabled), divider)
    });
    let rect = screen.get_by_role(Role::Splitter, "Split").rect().unwrap();
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 20.0, y);
    disabled.set(true);
    screen.run_until_idle();
    assert_eq!(*log.borrow(), [(Begin, 40.0), (Commit, 60.0)]);
    assert_eq!(value.get_untracked(), 60.0, "the drag stops where it is");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(phases(&log), [Begin, Commit]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_observer_of_a_disposed_child_is_skipped_when_the_session_ends_an_edit() {
    let log = Log::default();
    let seen = log.clone();
    let made = Rc::new(RefCell::new(None));
    let slot = made.clone();
    let (screen, (value, outer, inner)) = mount_with(real_window(), move || {
        let value = signal(10.0);
        let outer = signal(true);
        let inner = signal(true);
        let (seen, slot) = (seen.clone(), slot.clone());
        let view = show(outer, move || {
            let edit = EditSession::new(move |initial| value.set(initial));
            *slot.borrow_mut() = Some(edit);
            let seen = seen.clone();
            // The observer belongs to the child branch, not to the session's.
            show(inner, move || {
                edit.on_edit(observer(value, &seen));
                node()
            })
        });
        ((value, outer, inner), view)
    });
    let edit: EditSession<f64> = made.borrow().expect("the session was made");
    edit.begin(10.0);
    value.set(20.0);
    assert_eq!(*log.borrow(), [(Begin, 10.0)]);
    // The child goes first: canceling reaches no observer and is no misuse.
    inner.set(false);
    screen.run_until_idle();
    assert!(edit.cancel());
    assert_eq!(
        value.get_untracked(),
        10.0,
        "the starting value is restored"
    );
    assert_eq!(*log.borrow(), [(Begin, 10.0)]);
    // The session's owner is cleaned up with its child's observer already gone.
    inner.set(true);
    screen.run_until_idle();
    edit.begin(10.0);
    outer.set(false);
    screen.run_until_idle();
    assert!(
        screen.errors().is_empty(),
        "disposing the owners reported {:?}",
        screen.errors()
    );
}
