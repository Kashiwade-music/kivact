//! Role: Disabled custom controls obey the same input contract as native controls.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::style::Overflow;
use kivact::test::render;
use kivact::view::{Handled, PointerEvent};
use kivact::view::{MouseButton, PointerPhase};

#[test]
fn disabled_raw_handlers_keep_hover_but_take_no_pointer_wheel_or_key_edits() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let out = events.clone();
    let screen = render(move || {
        let (pointer, wheel, key, release, child) = (
            out.clone(),
            out.clone(),
            out.clone(),
            out.clone(),
            out.clone(),
        );
        column((node()
            .test_id("disabled")
            .disabled(true)
            .on_pointer(move |e: PointerEvent| {
                pointer.borrow_mut().push(format!("{:?}", e.phase));
                Handled::Yes
            })
            .on_wheel(move |_| {
                wheel.borrow_mut().push("wheel".into());
                Handled::Yes
            })
            .on_key(move |_| {
                key.borrow_mut().push("key".into());
                Handled::Yes
            })
            .on_key_up(move |_| {
                release.borrow_mut().push("release".into());
                Handled::Yes
            })
            .style(style().size(100, 100))
            .child(
                node()
                    .role(Role::Button)
                    .name("enabled child")
                    .focusable(true)
                    .test_id("child")
                    .on_pointer(move |e: PointerEvent| {
                        if e.phase == PointerPhase::Down {
                            child.borrow_mut().push("child".into());
                        }
                        Handled::Yes
                    })
                    .style(style().size(40, 40)),
            ),))
        .style(style().align_items(Align::Start))
    });
    screen.mouse_move(75.0, 75.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(80.0, 75.0);
    screen.mouse_up(MouseButton::Primary);
    screen.get_by_test_id("disabled").scroll(0.0, 20.0);
    screen.get_by_test_id("child").focus();
    screen.press_key(Key::ArrowDown);
    screen.mouse_leave();
    assert_eq!(
        *events.borrow(),
        ["Enter", "Leave"],
        "hover remains available for explanations"
    );
    screen.get_by_test_id("child").click();
    assert!(
        events.borrow().iter().any(|e| e == "child"),
        "disabled applies to the node, not its children"
    );
}

#[test]
fn disabling_a_captured_control_cancels_its_drag_before_it_can_resume() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let out = events.clone();
    let (screen, disabled) = mount(move || {
        let disabled = signal(false);
        (
            disabled,
            node()
                .test_id("drag")
                .disabled(disabled)
                .on_pointer(move |e: PointerEvent| {
                    out.borrow_mut().push(e.phase);
                    Handled::Yes
                })
                .style(style().size(100, 100)),
        )
    });
    screen.mouse_move(20.0, 20.0);
    screen.mouse_down(MouseButton::Primary);
    events.borrow_mut().clear();
    disabled.set(true);
    screen.run_until_idle();
    assert_eq!(
        *events.borrow(),
        [PointerPhase::Cancel],
        "disable ends an existing capture immediately"
    );
    screen.mouse_move(150.0, 150.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        *events.borrow(),
        [PointerPhase::Cancel, PointerPhase::Leave]
    );
    disabled.set(false);
    screen.run_until_idle();
    screen.mouse_move(20.0, 20.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert!(
        events
            .borrow()
            .ends_with(&[PointerPhase::Down, PointerPhase::Up]),
        "a new enabled press works normally"
    );
}

#[test]
fn a_disabled_wheel_listener_does_not_block_the_scroll_container() {
    let screen = render(|| {
        column((
            node()
                .test_id("off")
                .disabled(true)
                .on_wheel(|_| -> Handled { panic!("disabled wheel callback") })
                .style(style().size(100, 40).shrink(0)),
            node().style(style().size(100, 400).shrink(0)),
        ))
        .style(style().size(120, 100).overflow(Overflow::Scroll))
    });
    let before = screen.get_by_test_id("off").rect().expect("layout");
    screen.get_by_test_id("off").scroll(0.0, 40.0);
    let after = screen.get_by_test_id("off").rect().expect("layout");
    assert!(
        after.y < before.y,
        "unhandled wheel continues native scrolling: {before:?} {after:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
