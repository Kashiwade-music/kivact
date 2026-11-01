//! Role: Mouse capture regressions using only APIs available on baseline main.

use kivact::primitive::NodeExt;
use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::draw::Point;
use kivact::prelude::*;
use kivact::style::Overflow;
use kivact::test::{RenderOptions, WindowMode};
use kivact::view::{Handled, Placement, PointerEvent, overlay};
use kivact::view::{MouseButton, PointerKind, PointerPhase, ScrollBehavior};

type Events = Rc<RefCell<Vec<(&'static str, PointerEvent)>>>;

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
}

#[test]
fn mouse_button_chords_keep_the_first_listener_until_every_button_releases() {
    let events: Events = Rc::default();
    let output = events.clone();
    let (screen, ()) = mount_with(real_window(), move || {
        let first = output.clone();
        (
            (),
            column((
                node()
                    .test_id("first")
                    .style(style().size(100, 80))
                    .on_pointer(move |event| {
                        first.borrow_mut().push(("first", event));
                        Handled::No
                    }),
                node()
                    .test_id("second")
                    .style(style().size(100, 80))
                    .on_pointer(move |event| {
                        output.borrow_mut().push(("second", event));
                        Handled::No
                    }),
            ))
            .style(style().gap(20)),
        )
    });
    let first = screen.get_by_test_id("first").rect().unwrap();
    let second = screen.get_by_test_id("second").rect().unwrap();
    let outside = Point::new(400.0, 300.0);
    for (initial, additional, released, remaining) in [
        (
            MouseButton::Primary,
            MouseButton::Secondary,
            MouseButton::Secondary,
            MouseButton::Primary,
        ),
        (
            MouseButton::Secondary,
            MouseButton::Primary,
            MouseButton::Secondary,
            MouseButton::Primary,
        ),
        (
            MouseButton::Secondary,
            MouseButton::Primary,
            MouseButton::Primary,
            MouseButton::Secondary,
        ),
        (
            MouseButton::Primary,
            MouseButton::Middle,
            MouseButton::Primary,
            MouseButton::Middle,
        ),
    ] {
        events.borrow_mut().clear();
        screen.mouse_move(first.x + 20.0, first.y + 20.0);
        screen.mouse_down(initial);
        screen.mouse_move(second.x + 20.0, second.y + 20.0);
        screen.mouse_down(additional);
        screen.mouse_up(released);
        screen.mouse_move(outside.x, outside.y);
        screen.mouse_up(remaining);
        let accepted = events.borrow();
        let buttons = accepted
            .iter()
            .filter(|(_, event)| matches!(event.phase, PointerPhase::Down | PointerPhase::Up))
            .map(|(owner, event)| (*owner, event.phase, event.button))
            .collect::<Vec<_>>();
        assert_eq!(
            buttons,
            [
                ("first", PointerPhase::Down, Some(initial)),
                ("first", PointerPhase::Down, Some(additional)),
                ("first", PointerPhase::Up, Some(released)),
                ("first", PointerPhase::Up, Some(remaining)),
            ],
            "capture follows the first listener for {initial:?} then {additional:?}"
        );
        assert!(accepted.iter().any(|(owner, event)| {
            *owner == "first"
                && event.phase == PointerPhase::Move
                && event.window_pos == outside
                && event.button.is_none()
                && event.buttons.contains(remaining)
        }));
        assert!(
            !accepted
                .iter()
                .any(|(_, event)| event.phase == PointerPhase::Cancel)
        );
        drop(accepted);
        events.borrow_mut().clear();
        screen.mouse_move(outside.x + 10.0, outside.y);
        assert!(
            !events
                .borrow()
                .iter()
                .any(|(_, event)| event.phase == PointerPhase::Move),
            "the final release frees capture"
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disable_preserves_physical_buttons_and_lost_capture_clears_them() {
    let events: Events = Rc::default();
    let output = events.clone();
    let (screen, disabled) = mount_with(real_window(), move || {
        let disabled = signal(false);
        (
            disabled,
            node()
                .test_id("capture")
                .disabled(disabled)
                .style(style().size(100, 80))
                .on_pointer(move |event| {
                    output.borrow_mut().push(("capture", event));
                    Handled::No
                }),
        )
    });
    let rect = screen.get_by_test_id("capture").rect().unwrap();
    let (x, y) = (rect.x + 20.0, rect.y + 20.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_down(MouseButton::Secondary);
    disabled.set(true);
    screen.run_until_idle();
    assert_eq!(
        events
            .borrow()
            .iter()
            .filter(|(_, event)| event.phase == PointerPhase::Cancel)
            .count(),
        1
    );
    disabled.set(false);
    screen.run_until_idle();
    screen.mouse_up(MouseButton::Primary);
    events.borrow_mut().clear();
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_up(contact);
    assert!(
        events.borrow().iter().any(|(_, event)| {
            event.kind == PointerKind::Touch && event.phase == PointerPhase::Down
        }),
        "a contact is a pointer of its own, whatever mouse button is held"
    );
    screen.mouse_up(MouseButton::Secondary);
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_cancel(contact);
    assert!(events.borrow().iter().any(|(_, event)| {
        event.kind == PointerKind::Touch && event.phase == PointerPhase::Cancel
    }));
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Middle);
    screen.set_window_focused(false);
    screen.set_window_focused(true);
    events.borrow_mut().clear();
    screen.mouse_move(x + 1.0, y);
    screen.mouse_move(x + 2.0, y);
    let moves: Vec<_> = events
        .borrow()
        .iter()
        .filter(|(_, event)| event.phase == PointerPhase::Move)
        .map(|(_, event)| event.button)
        .collect();
    assert_eq!(
        moves,
        [None],
        "window capture loss clears the complete mouse-button ledger"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_secondary_release_does_not_end_a_native_primary_scrollbar_drag() {
    let (screen, (node_ref, offset)) = mount_with(real_window(), || {
        let node_ref = node_ref();
        let offset = node_ref.scroll_offset();
        (
            (node_ref, offset),
            node()
                .node_ref(node_ref)
                .test_id("scroll")
                .style(
                    style()
                        .size(100, 100)
                        .overflow(Overflow::Scroll)
                        .scrollbar_width(20),
                )
                .child(node().style(style().size(90, 500).shrink(0.0))),
        )
    });
    let rect = screen.get_by_test_id("scroll").rect().unwrap();
    let x = rect.x + 90.0;
    for secondary_first in [false, true] {
        node_ref.scroll_to(Point::new(0.0, 0.0), ScrollBehavior::Instant);
        screen.run_until_idle();
        screen.mouse_move(x, rect.y + 8.0);
        if secondary_first {
            screen.mouse_down(MouseButton::Secondary);
        }
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(x, rect.y + 30.0);
        let before = offset.get_untracked().y;
        if !secondary_first {
            screen.mouse_down(MouseButton::Secondary);
        }
        screen.mouse_up(MouseButton::Secondary);
        screen.mouse_move(x, rect.y + 60.0);
        assert!(
            offset.get_untracked().y > before,
            "secondary release keeps the primary scrollbar drag"
        );
        screen.mouse_down(MouseButton::Secondary);
        screen.mouse_up(MouseButton::Primary);
        let stopped = offset.get_untracked();
        screen.mouse_move(x, rect.y + 90.0);
        assert_eq!(
            offset.get_untracked(),
            stopped,
            "primary release ends the scrollbar drag"
        );
        screen.mouse_up(MouseButton::Secondary);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_button_chord_started_before_a_modal_keeps_its_outside_terminal_events() {
    let events: Events = Rc::default();
    let output = events.clone();
    let (screen, open) = mount_with(real_window(), move || {
        let open = signal(false);
        let anchor = node_ref();
        (
            open,
            (
                node()
                    .node_ref(anchor)
                    .test_id("before-modal")
                    .style(style().size(100, 80))
                    .on_pointer(move |event| {
                        output.borrow_mut().push(("before-modal", event));
                        Handled::No
                    }),
                show(open, move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        node().style(style().size(120, 80)),
                    )
                    .modal(true)
                    .role(Role::Dialog)
                    .name("Modal")
                }),
            ),
        )
    });
    let rect = screen.get_by_test_id("before-modal").rect().unwrap();
    let outside = Point::new(400.0, 300.0);
    for (initial, additional) in [
        (MouseButton::Primary, MouseButton::Secondary),
        (MouseButton::Secondary, MouseButton::Primary),
    ] {
        events.borrow_mut().clear();
        screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
        screen.mouse_down(initial);
        open.set(true);
        screen.run_until_idle();
        assert!(screen.query_by_role(Role::Dialog, "Modal").is_some());
        screen.mouse_down(additional);
        screen.mouse_up(initial);
        screen.mouse_move(outside.x, outside.y);
        screen.mouse_up(additional);
        let accepted = events.borrow();
        let buttons = accepted
            .iter()
            .filter(|(_, event)| matches!(event.phase, PointerPhase::Down | PointerPhase::Up))
            .map(|(_, event)| (event.phase, event.button))
            .collect::<Vec<_>>();
        assert_eq!(
            buttons,
            [
                (PointerPhase::Down, Some(initial)),
                (PointerPhase::Down, Some(additional)),
                (PointerPhase::Up, Some(initial)),
                (PointerPhase::Up, Some(additional)),
            ],
            "the original outside listener finishes every admitted press"
        );
        assert!(accepted.iter().any(|(_, event)| {
            event.phase == PointerPhase::Move
                && event.window_pos == outside
                && event.button.is_none()
                && event.buttons.contains(additional)
        }));
        assert!(
            !accepted
                .iter()
                .any(|(_, event)| event.phase == PointerPhase::Cancel)
        );
        drop(accepted);
        open.set(false);
        screen.run_until_idle();
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
