//! Role: Shared scalar semantics and real-window pointer drag lifetimes.

use kivact::primitive::NodeExt;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use crate::support::mount_with;
use kivact::binding;
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, use_drag, use_scalar};
use kivact::prelude::*;
use kivact::test::{RenderOptions, WindowMode};
use kivact::ui::ScalarRange;
use kivact::view::{A11yAction, Handled, KeyEvent};
use kivact::view::{Modifiers, MouseButton, PointerKind};

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
}

#[test]
fn a_scalar_uses_one_quantized_writer_for_keys_accessibility_and_dragging() {
    let (screen, (value, state)) = mount_with(real_window(), || {
        let value = signal(4.0);
        let range = ScalarRange::new(0.0, 10.0).unwrap().with_step(2.0).unwrap();
        let control = use_scalar(value, range);
        let origin = signal(0.0);
        let pointer = use_drag::<()>(DragCoordinates::Local)
            .on_start(move |_| origin.set(value.get_untracked()))
            .on_update(move |update: DragUpdate<()>| {
                control.set(origin.get_untracked() + f64::from(update.delta.x) / 10.0);
            });
        let part = control.thumb();
        let state = part.state();
        let view = node()
            .part(part)
            .part(pointer.source(()))
            .name("Level")
            .test_id("level")
            .style(style().size(100, 40))
            .on_key(move |event: KeyEvent| {
                if event.key == Key::ArrowRight && event.modifiers.shift {
                    control.step_by(2.0);
                    Handled::Yes
                } else {
                    Handled::No
                }
            });
        ((value, state), view)
    });
    let level = screen.get_by_role(Role::Slider, "Level");
    level.focus();
    screen.press_key_with(
        Key::ArrowRight,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(
        value.get_untracked(),
        8.0,
        "custom key stops the default increment"
    );
    screen.press_key(Key::ArrowRight);
    assert_eq!(
        value.get_untracked(),
        10.0,
        "unhandled custom key continues to scalar default"
    );
    screen.press_key(Key::Home);
    assert_eq!(value.get_untracked(), 0.0);
    assert_eq!(state.get_untracked().fraction, 0.0);
    screen.press_key(Key::End);
    assert_eq!(value.get_untracked(), 10.0);
    assert_eq!(state.get_untracked().fraction, 1.0);
    assert!(state.get_untracked().input.focused);
    screen.press_key(Key::PageDown);
    assert_eq!(value.get_untracked(), 0.0, "a page is ten steps");
    screen.press_key(Key::PageUp);
    assert_eq!(value.get_untracked(), 10.0);
    level.a11y_with_value(A11yAction::SetValue, "3");
    assert_eq!(level.value_range(), (Some(0.0), Some(10.0), Some(4.0)));
    level.a11y(A11yAction::Increment);
    assert_eq!(value.get_untracked(), 6.0);
    level.a11y(A11yAction::Decrement);
    assert_eq!(value.get_untracked(), 4.0);
    level.a11y_with_value(A11yAction::SetValue, "NaN");
    level.a11y_with_value(A11yAction::SetValue, "inf");
    assert_eq!(
        value.get_untracked(),
        4.0,
        "non-finite AT input does not write"
    );
    let rect = level.rect().unwrap();
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(rect.x + 55.0, rect.y + 20.0);
    assert_eq!(
        value.get_untracked(),
        8.0,
        "drag uses the same quantized setter"
    );
    value.set(0.0);
    screen.run_until_idle();
    screen.mouse_move(rect.x + 60.0, rect.y + 20.0);
    assert_eq!(
        value.get_untracked(),
        8.0,
        "drag keeps its initial value across external updates"
    );
    screen.mouse_up(MouseButton::Primary);
    value.set(0.0);
    screen.run_until_idle();
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(rect.x + 80.0, rect.y + 20.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        value.get_untracked(),
        2.0,
        "a new drag takes a fresh initial value"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn continuous_pointer_values_have_an_independent_key_step_and_equal_bounds_are_safe() {
    let (screen, (continuous, fixed)) = mount_with(real_window(), || {
        let continuous = signal(0.5);
        let fixed = signal(4.0);
        let control = use_scalar(continuous, ScalarRange::new(0.0, 1.0).unwrap())
            .key_step(0.25)
            .unwrap();
        let follow = move |update: DragUpdate<()>| {
            control.set(f64::from(update.pos.x) / 100.0);
        };
        let pointer = use_drag::<()>(DragCoordinates::Local)
            .on_start(follow)
            .on_update(follow);
        let fixed_control = use_scalar(fixed, ScalarRange::new(4.0, 4.0).unwrap());
        (
            (continuous, fixed),
            column((
                node()
                    .part(control.thumb())
                    .part(pointer.source(()))
                    .name("Continuous")
                    .style(style().size(100, 40)),
                node()
                    .part(fixed_control.thumb())
                    .name("Fixed")
                    .style(style().size(100, 40)),
            ))
            .style(style().gap(20)),
        )
    });
    let continuous_view = screen.get_by_role(Role::Slider, "Continuous");
    let rect = continuous_view.rect().unwrap();
    screen.click_at(rect.x + 37.0, rect.y + 20.0);
    assert!(
        (continuous.get_untracked() - 0.37).abs() < 0.000001,
        "pointer values are not quantized"
    );
    continuous_view.focus();
    screen.press_key(Key::ArrowRight);
    assert!((continuous.get_untracked() - 0.62).abs() < 0.000001);
    continuous_view.a11y(A11yAction::Increment);
    assert!((continuous.get_untracked() - 0.87).abs() < 0.000001);
    let fixed_view = screen.get_by_role(Role::Slider, "Fixed");
    fixed_view.focus();
    screen.press_key(Key::End);
    screen.press_key(Key::ArrowLeft);
    fixed_view.a11y_with_value(A11yAction::SetValue, "200");
    assert_eq!(fixed.get_untracked(), 4.0);
    assert_eq!(fixed_view.value_range(), (Some(4.0), Some(4.0), Some(4.0)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_controlled_scalar_offers_writes_without_claiming_binding_acceptance() {
    let (screen, (value, offers, control)) = mount_with(real_window(), || {
        let value = signal(4.0);
        let offers = signal(0usize);
        let binding = binding(value.into_value(), move |candidate: f64| {
            offers.update(|offers| *offers += 1);
            if candidate <= 6.0 {
                value.set(candidate);
            }
        });
        let control = use_scalar(binding, ScalarRange::new(0.0, 10.0).unwrap());
        (
            (value, offers, control),
            node()
                .part(control.thumb())
                .name("Controlled")
                .style(style().size(100, 40)),
        )
    });
    let slider = screen.get_by_role(Role::Slider, "Controlled");
    assert!(
        control.set(9.0),
        "true means the writer was offered a value"
    );
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), 4.0);
    assert_eq!(offers.get_untracked(), 1);
    assert_eq!(slider.value_range(), (Some(0.0), Some(10.0), Some(4.0)));
    slider.a11y_with_value(A11yAction::SetValue, "9");
    assert_eq!(value.get_untracked(), 4.0);
    assert_eq!(offers.get_untracked(), 2);
    slider.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 5.0);
    assert_eq!(offers.get_untracked(), 3);
    assert_eq!(slider.value_range(), (Some(0.0), Some(10.0), Some(5.0)));
    assert!(!control.set(f64::NAN));
    assert_eq!(
        offers.get_untracked(),
        3,
        "invalid input never calls the writer"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_drag_starts_once_ignores_secondary_buttons_and_ends_on_pen_cancel_or_release() {
    let starts = Rc::new(Cell::new(0));
    let events = Rc::new(RefCell::new(Vec::new()));
    let out_starts = starts.clone();
    let out_events = events.clone();
    let (screen, (drag, state)) = mount_with(real_window(), move || {
        let record = move |update: DragUpdate<f64>| {
            if update.phase == DragPhase::Start {
                out_starts.set(out_starts.get() + 1);
            }
            out_events
                .borrow_mut()
                .push((update.phase, update.pointer, update.delta));
        };
        let record = Rc::new(record);
        let (on_start, on_update, on_end) = (record.clone(), record.clone(), record);
        let session = use_drag(DragCoordinates::Window)
            .on_start(move |update| on_start(update))
            .on_update(move |update| on_update(update))
            .on_end(move |update| on_end(update));
        let source = session.source(10.0);
        let state = source.state();
        let view = node()
            .part(source)
            .test_id("drag")
            .style(style().size(100, 80));
        ((session, state), view)
    });
    let rect = screen.get_by_test_id("drag").rect().unwrap();
    let (x, y) = (rect.x + 20.0, rect.y + 20.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Secondary);
    assert_eq!(starts.get(), 0);
    screen.mouse_down(MouseButton::Primary);
    assert!(drag.is_active());
    assert!(
        state.get_untracked().active,
        "the source shows its active drag"
    );
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Secondary);
    assert!(
        drag.is_active(),
        "secondary release cannot end the primary drag"
    );
    screen.mouse_move(x + 120.0, y + 10.0);
    assert_eq!(
        starts.get(),
        1,
        "moving outside uses existing capture without a new start"
    );
    screen.mouse_up(MouseButton::Primary);
    assert!(!drag.is_active());
    assert!(!state.get_untracked().active);
    let contact = screen.contact_down(PointerKind::Pen, x, y);
    screen.contact_move(contact, x + 10.0, y);
    screen.contact_cancel(contact);
    assert!(!drag.is_active());
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_up(contact);
    assert!(!drag.is_active());
    assert_eq!(starts.get(), 3);
    let events = events.borrow();
    let identities = events
        .iter()
        .filter(|e| e.0 == DragPhase::Start)
        .map(|e| e.1)
        .collect::<Vec<_>>();
    assert_eq!(identities.len(), 3);
    assert_ne!(identities[0], identities[1]);
    assert_ne!(identities[1], identities[2]);
    assert!(events.iter().any(|e| e.0 == DragPhase::Cancel));
    assert_eq!(events.iter().filter(|e| e.0 == DragPhase::End).count(), 2);
    assert!(
        events.iter().any(|e| e.2.x >= 120.0),
        "captured movement reaches outside the node"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disabling_and_unmounting_discard_the_owned_start_value() {
    let weak: Rc<RefCell<Weak<()>>> = Rc::default();
    let out = weak.clone();
    let (screen, (disabled, visible)) = mount_with(real_window(), move || {
        let disabled = signal(false);
        let visible = signal(true);
        (
            (disabled, visible),
            show(visible, move || {
                let session = use_drag(DragCoordinates::Window).disabled(disabled);
                // The source holds one reference; an active drag holds another.
                let payload = Rc::new(());
                *out.borrow_mut() = Rc::downgrade(&payload);
                node()
                    .part(session.source(payload))
                    .test_id("owned-drag")
                    .style(style().size(100, 80))
            }),
        )
    });
    let rect = screen.get_by_test_id("owned-drag").rect().unwrap();
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    let held = || weak.borrow().strong_count();
    assert_eq!(held(), 1, "only the source holds its payload");
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(held(), 2, "the active drag captured the payload");
    disabled.set(true);
    screen.run_until_idle();
    assert_eq!(
        held(),
        1,
        "the separate disabled source releases the captured start"
    );
    screen.mouse_up(MouseButton::Primary);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(
        held(),
        1,
        "disabled sessions refuse a new start even on an enabled node"
    );
    screen.mouse_up(MouseButton::Primary);
    disabled.set(false);
    screen.run_until_idle();
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(held(), 2);
    visible.set(false);
    screen.run_until_idle();
    assert_eq!(held(), 0, "branch owner disposal drops drag state");
    screen.mouse_up(MouseButton::Primary);
    visible.set(true);
    screen.run_until_idle();
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(
        held(),
        2,
        "a remounted session starts with fresh owner state"
    );
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(held(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn engine_disabled_capture_cancels_a_drag_without_an_extra_disabled_subscription() {
    let (screen, (disabled, drag)) = mount_with(real_window(), || {
        let disabled = signal(false);
        let session = use_drag(DragCoordinates::Window);
        (
            (disabled, session),
            node()
                .part(session.source(0.0))
                .test_id("native-disabled-drag")
                .disabled(disabled)
                .style(style().size(100, 80)),
        )
    });
    let rect = screen
        .get_by_test_id("native-disabled-drag")
        .rect()
        .unwrap();
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    screen.mouse_down(MouseButton::Primary);
    assert!(drag.is_active());
    disabled.set(true);
    screen.run_until_idle();
    assert!(
        !drag.is_active(),
        "native capture cancellation reaches the existing pointer handler"
    );
    screen.mouse_move(rect.x + 50.0, rect.y + 20.0);
    screen.mouse_up(MouseButton::Primary);
    disabled.set(false);
    screen.run_until_idle();
    screen.mouse_down(MouseButton::Primary);
    assert!(drag.is_active());
    screen.mouse_up(MouseButton::Primary);
    assert!(!drag.is_active());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn spin_buttons_keep_native_carets_and_share_quantized_accessibility_writes() {
    let (screen, (number, plain)) = mount_with(real_window(), || {
        let number = signal(123.0_f64);
        let plain = signal("123".to_owned());
        (
            (number, plain),
            column((
                kivact::ui::number_input(
                    "Count",
                    number,
                    ScalarRange::new(1.0, 200.0)
                        .and_then(|range| range.with_step(1.0))
                        .expect("invariant: the bounds and the step are finite"),
                ),
                kivact::ui::text_input("Plain", plain),
                kivact::ui::text_input("Decorated", plain).role(Role::SpinButton),
            )),
        )
    });
    let spin = screen.get_by_role(Role::SpinButton, "Count");
    spin.focus();
    for (key, selection) in [(Key::ArrowLeft, (2, 2)), (Key::ArrowRight, (3, 3))] {
        screen.press_key(key);
        assert_eq!(spin.text_selection(), Some(selection));
        assert_eq!(number.get_untracked(), 123.0);
    }
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    screen.press_key_with(Key::Home, shift);
    assert_eq!(spin.text_selection(), Some((3, 0)));
    screen.press_key_with(Key::Home, shift);
    assert_eq!(
        number.get_untracked(),
        123.0,
        "a native no-op cannot leak to scalar keys"
    );
    screen.press_key_with(Key::End, shift);
    assert_eq!(spin.text_selection(), Some((3, 3)));
    screen.press_key_with(Key::End, shift);
    assert_eq!(number.get_untracked(), 123.0);
    screen.press_key(Key::ArrowLeft);
    screen.press_key(Key::Home);
    assert_eq!(number.get_untracked(), 1.0);
    screen.press_key(Key::End);
    assert_eq!(number.get_untracked(), 200.0);
    for (text, expected) in [
        ("5.0", 5.0),
        ("5.5", 6.0),
        ("NaN", 6.0),
        ("inf", 6.0),
        ("9999999999", 200.0),
        ("-9999999999", 1.0),
    ] {
        spin.a11y_with_value(A11yAction::SetValue, text);
        assert_eq!(number.get_untracked(), expected);
        assert_eq!(spin.value(), Some(expected.to_string()));
        assert_eq!(spin.value_range(), (Some(1.0), Some(200.0), Some(expected)));
    }
    for (outside, action, expected) in [
        (300.0, A11yAction::Decrement, 199.0),
        (-20.0, A11yAction::Increment, 2.0),
    ] {
        number.set(outside);
        screen.run_until_idle();
        assert_eq!(
            number.get_untracked(),
            outside,
            "constant bounds do not rewrite external values"
        );
        spin.a11y(action);
        assert_eq!(
            number.get_untracked(),
            expected,
            "step from the normalized current value"
        );
    }
    let field = screen.get_by_role(Role::TextBox, "Plain");
    field.focus();
    screen.press_key(Key::Home);
    assert_eq!(field.text_selection(), Some((0, 0)));
    screen.press_key(Key::End);
    assert_eq!(field.text_selection(), Some((3, 3)));
    field.a11y_with_value(A11yAction::SetValue, "5.5");
    assert_eq!(plain.get_untracked(), "5.5");
    let decorated = screen.get_by_role(Role::SpinButton, "Decorated");
    decorated.a11y_with_value(A11yAction::SetValue, "7.5");
    assert_eq!(
        plain.get_untracked(),
        "7.5",
        "a role override retains native SetValue"
    );
    decorated.focus();
    screen.press_key(Key::ArrowLeft);
    screen.press_key(Key::Home);
    assert_eq!(decorated.text_selection(), Some((0, 0)));
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::End);
    assert_eq!(decorated.text_selection(), Some((3, 3)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_moving_range_writes_the_value_back_once_however_many_parts_are_attached() {
    let (screen, (bounds, value, writes)) = mount_with(real_window(), || {
        let bounds = signal(ScalarRange::new(0.0, 100.0).unwrap());
        let value = signal(50.0);
        let writes = signal(0usize);
        let counted = binding(value.into_value(), move |next: f64| {
            writes.update(|writes| *writes += 1);
            value.set(next);
        });
        let control = use_scalar(counted, bounds);
        // Both parts are made; the one node can take only one of them.
        let (thumb, input) = (control.thumb(), control.input());
        let _ = input;
        let view = node().part(thumb).name("Thumb");
        ((bounds, value, writes), view)
    });
    bounds.set(ScalarRange::new(0.0, 10.0).unwrap());
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), 10.0);
    assert_eq!(
        writes.get_untracked(),
        1,
        "one effect per scalar, not one per part"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
