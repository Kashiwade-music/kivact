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
use kivact::ui::{ScalarRange, ScalarRangeError};
use kivact::view::{A11yAction, Handled, KeyEvent};
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
fn stacked_panes_grow_downward_and_leave_the_horizontal_arrows_alone() {
    let keys = Rc::new(RefCell::new(Vec::new()));
    let seen = keys.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(40.0);
        let divider =
            handle("Stack").part(use_splitter(value, range(), Orientation::Vertical).handle());
        (value, recorded(divider, seen))
    });
    let split = screen.get_by_role(Role::Splitter, "Stack");
    split.focus();
    screen.press_key(Key::ArrowDown);
    assert_eq!(
        value.get_untracked(),
        41.0,
        "the default step is 1% of the range"
    );
    screen.press_key(Key::ArrowUp);
    screen.press_key(Key::ArrowUp);
    assert_eq!(value.get_untracked(), 39.0);
    screen.press_key(Key::ArrowLeft);
    screen.press_key(Key::ArrowRight);
    assert_eq!(
        value.get_untracked(),
        39.0,
        "horizontal arrows are not used"
    );
    assert_eq!(*keys.borrow(), vec![Key::ArrowLeft, Key::ArrowRight]);
    screen.press_key(Key::End);
    assert_eq!(value.get_untracked(), 100.0);
    screen.press_key(Key::Home);
    assert_eq!(value.get_untracked(), 0.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_stepped_range_sets_the_default_key_step_and_an_invalid_step_is_rejected() {
    let (screen, value) = mount_with(real_window(), || {
        let value = signal(10.0);
        let stepped = ScalarRange::new(0.0, 100.0)
            .unwrap()
            .with_step(20.0)
            .unwrap();
        let handle_of = use_splitter(value, stepped, Orientation::Horizontal);
        for step in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                handle_of.key_step(step).err(),
                Some(ScalarRangeError::InvalidStep)
            );
        }
        (value, handle("Stepped").part(handle_of.handle()))
    });
    let split = screen.get_by_role(Role::Splitter, "Stepped");
    split.press_key(Key::ArrowRight);
    assert_eq!(
        value.get_untracked(),
        40.0,
        "10 is quantized to 20, then one step of 20"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn enter_and_double_click_move_to_the_reset_value_only_when_there_is_one() {
    let (screen, (value, reset)) = mount_with(real_window(), || {
        let value = signal(80.0);
        let reset = signal(Some(25.0));
        let divider = handle("Resettable").part(
            use_splitter(value, range(), Orientation::Horizontal)
                .reset_to(reset)
                .handle(),
        );
        ((value, reset), divider)
    });
    let split = screen.get_by_role(Role::Splitter, "Resettable");
    split.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 25.0);
    value.set(80.0);
    screen.run_until_idle();
    split.double_click();
    assert_eq!(value.get_untracked(), 25.0);
    value.set(80.0);
    reset.set(None);
    screen.run_until_idle();
    split.press_key(Key::Enter);
    split.double_click();
    assert_eq!(
        value.get_untracked(),
        80.0,
        "no reset value, nothing happens"
    );
    reset.set(Some(500.0));
    screen.run_until_idle();
    split.press_key(Key::Enter);
    assert_eq!(
        value.get_untracked(),
        100.0,
        "the reset goes through the range"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn assistive_technology_sets_increments_and_decrements_through_the_range() {
    let (screen, value) = mount_with(real_window(), || {
        let value = signal(50.0);
        let divider = handle("AT").part(
            use_splitter(value, range(), Orientation::Vertical)
                .key_step(10.0)
                .unwrap()
                .handle(),
        );
        (value, divider)
    });
    let split = screen.get_by_role(Role::Splitter, "AT");
    split.a11y(A11yAction::Increment);
    assert_eq!(value.get_untracked(), 60.0);
    split.a11y(A11yAction::Decrement);
    split.a11y(A11yAction::Decrement);
    assert_eq!(value.get_untracked(), 40.0);
    split.a11y_with_value(A11yAction::SetValue, "73");
    assert_eq!(value.get_untracked(), 73.0);
    split.a11y_with_value(A11yAction::SetValue, "500");
    assert_eq!(value.get_untracked(), 100.0);
    split.a11y_with_value(A11yAction::SetValue, "NaN");
    split.a11y_with_value(A11yAction::SetValue, "not a number");
    assert_eq!(value.get_untracked(), 100.0);
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

#[test]
fn stacked_panes_drag_along_y_with_a_ns_cursor() {
    let (screen, value) = mount_with(real_window(), || {
        let value = signal(10.0);
        let divider = node()
            .name("Rows")
            .style(style().size(100, 8).cursor(Cursor::NsResize))
            .part(use_splitter(value, range(), Orientation::Vertical).handle());
        (value, divider)
    });
    let split = screen.get_by_role(Role::Splitter, "Rows");
    let rect = split.rect().unwrap();
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    assert_eq!(screen.cursor(), Cursor::NsResize);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 40.0, y + 30.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(value.get_untracked(), 40.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_disabled_splitter_ignores_input_and_assistive_technology_and_cancels_a_drag() {
    let (screen, (value, disabled)) = mount_with(real_window(), || {
        let value = signal(40.0);
        let disabled = signal(false);
        let divider = handle("Gated").part(
            use_splitter(value, range(), Orientation::Horizontal)
                .reset_to(Some(10.0))
                .disabled(disabled)
                .handle(),
        );
        ((value, disabled), divider)
    });
    let split = screen.get_by_role(Role::Splitter, "Gated");
    let rect = split.rect().unwrap();
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    assert!(!split.is_disabled());
    split.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 41.0);
    assert!(split.is_focused());

    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 10.0, y);
    assert_eq!(value.get_untracked(), 51.0);
    disabled.set(true);
    screen.run_until_idle();
    screen.mouse_move(x + 30.0, y);
    assert_eq!(value.get_untracked(), 51.0, "disabling cancels the drag");
    screen.mouse_up(MouseButton::Primary);

    assert!(split.is_disabled());
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 20.0, y);
    screen.mouse_up(MouseButton::Primary);
    split.force_click();
    split.force_click();
    split.a11y(A11yAction::Increment);
    split.a11y_with_value(A11yAction::SetValue, "90");
    assert_eq!(value.get_untracked(), 51.0, "disabled: no pointer, no AT");
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::Home);
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 51.0, "disabled: no keys");

    disabled.set(false);
    screen.run_until_idle();
    assert!(!split.is_disabled());
    split.press_key(Key::End);
    assert_eq!(value.get_untracked(), 100.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn semantics_publish_the_role_the_range_the_value_and_the_separator_direction() {
    let (screen, value) = mount_with(real_window(), || {
        let value = signal(30.0);
        let range = ScalarRange::new(10.0, 90.0).unwrap();
        (
            value,
            column((
                handle("Side by side")
                    .part(use_splitter(value, range, Orientation::Horizontal).handle()),
                handle("Stacked").part(use_splitter(value, range, Orientation::Vertical).handle()),
            )),
        )
    });
    let side = screen.get_by_role(Role::Splitter, "Side by side");
    let stacked = screen.get_by_role(Role::Splitter, "Stacked");
    assert_eq!(side.role(), Some(Role::Splitter));
    assert_eq!(side.value_range(), (Some(10.0), Some(90.0), Some(30.0)));
    assert_eq!(
        side.orientation(),
        Orientation::Vertical,
        "panes side by side are separated by a vertical line"
    );
    assert_eq!(stacked.orientation(), Orientation::Horizontal);
    value.set(70.0);
    screen.run_until_idle();
    assert_eq!(stacked.value_range(), (Some(10.0), Some(90.0), Some(70.0)));
    assert!(
        side.a11y_actions().contains(&A11yAction::SetValue),
        "{:?}",
        side.a11y_actions()
    );
    side.focus();
    assert!(side.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_range_that_follows_the_layout_updates_the_semantics_keys_and_clamps_the_value() {
    let (screen, (value, bounds)) = mount_with(real_window(), || {
        let value = signal(60.0);
        let bounds = signal((10.0, 90.0));
        let divider = handle("Following").part(
            use_splitter(
                value,
                move || {
                    let (min, max) = bounds.get();
                    ScalarRange::new(min, max).expect("invariant: finite range")
                },
                Orientation::Horizontal,
            )
            .handle(),
        );
        ((value, bounds), divider)
    });
    let split = screen.get_by_role(Role::Splitter, "Following");
    assert_eq!(split.value_range(), (Some(10.0), Some(90.0), Some(60.0)));
    bounds.set((20.0, 80.0));
    screen.run_until_idle();
    assert_eq!(split.value_range(), (Some(20.0), Some(80.0), Some(60.0)));
    assert_eq!(value.get_untracked(), 60.0, "a value inside is not written");
    split.press_key(Key::End);
    assert_eq!(value.get_untracked(), 80.0);
    split.press_key(Key::Home);
    assert_eq!(value.get_untracked(), 20.0);
    split.a11y_with_value(A11yAction::SetValue, "95");
    assert_eq!(value.get_untracked(), 80.0, "SetValue follows the range");

    value.set(75.0);
    bounds.set((10.0, 50.0));
    screen.run_until_idle();
    assert_eq!(
        value.get_untracked(),
        50.0,
        "a value left outside is clamped"
    );
    assert_eq!(split.value_range(), (Some(10.0), Some(50.0), Some(50.0)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
