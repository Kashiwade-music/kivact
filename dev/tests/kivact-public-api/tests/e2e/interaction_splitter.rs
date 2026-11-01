//! Role: Tests for splitter interaction and accessibility.
//!
//! The unstyled `splitter` handle: keys, assistive technology, pointer drags and
//! semantics shared by every splitter of the official kits.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::interaction::use_splitter;
use kivact::prelude::*;
use kivact::style::Cursor;
use kivact::test::{RenderOptions, WindowMode};
use kivact::ui::ScalarRange;
use kivact::view::{Handled, KeyEvent};
use kivact::view::{MouseButton, Orientation};

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
}

fn range() -> ScalarRange {
    ScalarRange::new(0.0, 100.0).expect("invariant: finite range")
}

/// A splitter in a parent that records the keys no splitter handler consumed.
fn recorded(divider: Node, keys: Rc<RefCell<Vec<Key>>>) -> Node {
    node()
        .on_key(move |event: KeyEvent| {
            keys.borrow_mut().push(event.key);
            Handled::No
        })
        .child(divider)
}

fn handle(label: &str) -> Node {
    node().name(label).style(style().size(8, 100))
}

#[test]
fn side_by_side_panes_use_left_and_right_and_leave_the_vertical_arrows_alone() {
    let keys = Rc::new(RefCell::new(Vec::new()));
    let seen = keys.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(40.0);
        let divider = handle("Side").part(
            use_splitter(value, range(), Orientation::Horizontal)
                .key_step(5.0)
                .unwrap()
                .handle(),
        );
        (value, recorded(divider, seen))
    });
    let split = screen.get_by_role(Role::Splitter, "Side");
    split.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 45.0);
    screen.press_key(Key::ArrowLeft);
    screen.press_key(Key::ArrowLeft);
    assert_eq!(value.get_untracked(), 35.0);
    screen.press_key(Key::ArrowUp);
    screen.press_key(Key::ArrowDown);
    assert_eq!(value.get_untracked(), 35.0, "vertical arrows are not used");
    assert_eq!(
        *keys.borrow(),
        vec![Key::ArrowUp, Key::ArrowDown],
        "unused arrows bubble; handled keys stop"
    );
    screen.press_key(Key::Home);
    assert_eq!(value.get_untracked(), 0.0);
    screen.press_key(Key::ArrowLeft);
    assert_eq!(value.get_untracked(), 0.0, "clamped at the minimum");
    screen.press_key(Key::End);
    assert_eq!(value.get_untracked(), 100.0);
    screen.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 100.0, "clamped at the maximum");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_drag_moves_by_the_pointer_travel_times_the_scale_and_escape_restores_the_start() {
    let (screen, (value, scale, state)) = mount_with(real_window(), || {
        let value = signal(40.0);
        let scale = signal(1.0);
        let part = use_splitter(value, range(), Orientation::Horizontal)
            .pointer_scale(scale)
            .handle();
        let state = part.state();
        let divider = handle("Dragged")
            .style(style().cursor(Cursor::EwResize))
            .part(part);
        ((value, scale, state), divider)
    });
    let split = screen.get_by_role(Role::Splitter, "Dragged");
    let rect = split.rect().unwrap();
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    assert_eq!(screen.cursor(), Cursor::EwResize);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 20.0, y + 50.0);
    assert_eq!(
        value.get_untracked(),
        60.0,
        "only the travel along x counts, one unit per pixel"
    );
    assert!(state.get_untracked().dragging);
    assert_eq!(state.get_untracked().value, 60.0);
    scale.set(0.5);
    screen.run_until_idle();
    screen.mouse_move(x + 30.0, y);
    assert_eq!(value.get_untracked(), 55.0, "the scale follows its source");
    screen.mouse_move(x - 400.0, y);
    assert_eq!(value.get_untracked(), 0.0, "clamped by the range");
    screen.mouse_up(MouseButton::Primary);
    assert!(!state.get_untracked().dragging);

    scale.set(1.0);
    value.set(40.0);
    screen.run_until_idle();
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 25.0, y);
    assert_eq!(value.get_untracked(), 65.0);
    screen.press_key(Key::Escape);
    assert_eq!(
        value.get_untracked(),
        40.0,
        "Escape restores the starting value"
    );
    screen.mouse_move(x + 50.0, y);
    assert_eq!(value.get_untracked(), 40.0, "the drag ended");
    screen.mouse_up(MouseButton::Primary);
    screen.press_key(Key::Escape);
    assert_eq!(
        value.get_untracked(),
        40.0,
        "Escape without a drag does nothing"
    );

    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 10.0, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        value.get_untracked(),
        50.0,
        "a new drag starts from the current value"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
