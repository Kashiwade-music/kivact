//! Role: Stable gradient stop editing, transforms and exact cancellation.
use super::*;
use kivact::style::Color;
use kivact::ui::EditPhase::{Begin, Cancel, Commit};
use kivact::view::{PointerKind, callback};
fn stops() -> Vec<ui::GradientStop> {
    vec![
        ui::GradientStop::new(0.25, rgb(0x804020)),
        ui::GradientStop::new(0.75, rgb(0x204080)),
    ]
}

#[test]
fn gradient_constructor_empty_single_add_remove_and_index_selection() {
    assert_eq!(ui::GradientStop::new(f64::NAN, Color::WHITE).position, 0.0);
    assert_eq!(
        ui::GradientStop::new(f64::NEG_INFINITY, Color::WHITE).position,
        0.0
    );
    assert_eq!(
        ui::GradientStop::new(f64::INFINITY, Color::WHITE).position,
        1.0
    );
    assert_eq!(ui::GradientStop::new(-2.0, Color::WHITE).position, 0.0);
    assert_eq!(ui::GradientStop::new(2.0, Color::WHITE).position, 1.0);
    let (screen, (data, selected)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(Vec::<ui::GradientStop>::new());
        let selected = signal(None);
        (
            (data, selected),
            ui::gradient_editor(data, selected).name("Gradient"),
        )
    });
    screen.get_by_role(Role::Group, "Gradient");
    screen.get_by_role(Role::Canvas, "Gradient stops");
    assert!(
        screen
            .get_by_role(Role::Button, "Remove stop")
            .is_disabled()
    );
    screen.get_by_role(Role::Button, "Add stop").click();
    assert_eq!(selected.get_untracked(), Some(0));
    assert_eq!(
        data.get_untracked()[0],
        ui::GradientStop::new(0.5, Color::WHITE)
    );
    screen.get_by_role(Role::Button, "Remove stop").click();
    assert!(data.get_untracked().is_empty());
    assert_eq!(selected.get_untracked(), None);
    data.set(vec![
        ui::GradientStop::new(0.25, Color::BLACK),
        ui::GradientStop::new(0.75, Color::WHITE),
    ]);
    selected.set(Some(0));
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Add stop").click();
    assert_eq!(selected.get_untracked(), Some(2));
    assert_eq!(data.get_untracked()[2].color, Color::BLACK);
    screen.get_by_role(Role::Button, "Remove stop").click();
    assert_eq!(selected.get_untracked(), Some(1));
    let ramp = screen
        .get_by_role(Role::Canvas, "Gradient stops")
        .rect()
        .unwrap();
    screen.click_at(ramp.center().x, ramp.y + 10.0);
    assert_eq!(data.get_untracked().len(), 2);
    data.set(
        (0..300)
            .map(|_| ui::GradientStop::new(0.5, Color::WHITE))
            .collect(),
    );
    selected.set(None);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Add stop").click();
    assert_eq!(data.get_untracked().len(), 301);
    assert_eq!(selected.get_untracked(), Some(300));
    clean(&screen);
}

#[test]
fn gradient_keys_actions_selection_and_position_reorder_keep_storage_order() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (data, selected)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let data = signal(stops());
            let selected = signal(None);
            (
                (data, selected),
                ui::gradient_editor(data, selected)
                    .on_edit(move |phase| seen.borrow_mut().push(phase)),
            )
        });
    let stop = screen.get_by_role(Role::Slider, "Gradient stop 0");
    for action in [
        A11yAction::Select,
        A11yAction::Activate,
        A11yAction::SetValue,
        A11yAction::Increment,
        A11yAction::Decrement,
    ] {
        assert!(stop.a11y_actions().contains(&action));
    }
    stop.a11y(A11yAction::Select);
    assert_eq!(selected.get_untracked(), Some(0));
    assert!(stop.is_focused());
    stop.a11y(A11yAction::Activate);
    assert!(log.borrow().is_empty());
    stop.press_key(Key::ArrowRight);
    screen.press_key_with(
        Key::ArrowUp,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert!((data.get_untracked()[0].position - 0.36).abs() < 1e-9);
    stop.a11y_with_value(A11yAction::SetValue, "0.9");
    assert_eq!(selected.get_untracked(), Some(0));
    stop.a11y(A11yAction::Increment);
    stop.a11y(A11yAction::Decrement);
    stop.press_key(Key::Home);
    assert_eq!(data.get_untracked()[0].position, 0.0);
    stop.press_key(Key::End);
    assert_eq!(data.get_untracked()[0].position, 1.0);
    screen
        .get_by_role(Role::SpinButton, "Stop position")
        .a11y_with_value(A11yAction::SetValue, "0.6");
    assert_eq!(data.get_untracked()[0].position, 0.6);
    let count = log.borrow().len();
    stop.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(log.borrow().len(), count);
    stop.press_key(Key::Delete);
    assert_eq!(selected.get_untracked(), Some(0));
    assert_eq!(data.get_untracked().len(), 1);
    assert!(
        log.borrow()
            .as_chunks::<2>()
            .0
            .iter()
            .all(|p| *p == [Begin, Commit])
    );
    clean(&screen);
}

#[test]
fn gradient_stop_drags_and_all_cancel_paths_restore_exact_vector_and_selection() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (data, selected, shown, disabled, hidden)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let data = signal(stops());
            let selected = signal(Some(1));
            let shown = signal(true);
            let disabled = signal(false);
            let hidden = signal(false);
            let observer = callback(move |phase| {
                seen.borrow_mut()
                    .push((phase, data.get_untracked(), selected.get_untracked()))
            });
            (
                (data, selected, shown, disabled, hidden),
                show(shown, move || {
                    ui::gradient_editor(data, selected)
                        .disabled(disabled)
                        .hidden(hidden)
                        .on_edit(move |value| observer.try_call(value))
                }),
            )
        });
    for mode in 0..6 {
        log.borrow_mut().clear();
        let original = data.get_untracked();
        let initial_selection = selected.get_untracked();
        let stop = screen.get_by_role(Role::Slider, "Gradient stop 0");
        let p = stop.rect().unwrap().center();
        let ramp = screen
            .get_by_role(Role::Canvas, "Gradient stops")
            .rect()
            .unwrap();
        let contact = if mode == 2 {
            let contact = screen.contact_down(PointerKind::Touch, p.x, p.y);
            screen.contact_move(contact, p.x + 40.0, p.y);
            Some(contact)
        } else {
            screen.mouse_move(p.x, p.y);
            screen.mouse_down(MouseButton::Primary);
            screen.mouse_move(p.x + 40.0, p.y);
            None
        };
        assert_ne!(data.get_untracked(), original);
        assert_eq!(selected.get_untracked(), Some(0));
        let moved = data.get_untracked();
        stop.a11y_with_value(A11yAction::SetValue, "0.99");
        screen
            .get_by_role(Role::SpinButton, "Color R")
            .a11y_with_value(A11yAction::SetValue, "0.99");
        assert_eq!(data.get_untracked(), moved);
        match mode {
            0 => screen.mouse_up(MouseButton::Primary),
            1 => screen.press_key(Key::Escape),
            2 => screen.contact_cancel(contact.expect("the touch is down")),
            3 => {
                disabled.set(true);
                screen.run_until_idle();
            }
            4 => {
                hidden.set(true);
                screen.run_until_idle();
            }
            _ => {
                shown.set(false);
                screen.run_until_idle();
            }
        }
        let terminal = if mode == 0 { Commit } else { Cancel };

        if mode == 0 {
            assert!(
                (data.get_untracked()[0].position
                    - original[0].position
                    - 40.0 / f64::from(ramp.width - 16.0))
                .abs()
                    < 1e-6
            );
        } else {
            assert_eq!(data.get_untracked(), original);
            assert_eq!(selected.get_untracked(), initial_selection);
        }
        let last = log.borrow().last().cloned().unwrap();
        assert_eq!(last.1, data.get_untracked());
        assert_eq!(last.2, selected.get_untracked());
        if mode != 2 {
            screen.mouse_up(MouseButton::Primary);
        }
        disabled.set(false);
        hidden.set(false);
        screen.run_until_idle();
        screen.advance(Duration::from_secs(1));
        assert_eq!(
            log.borrow().iter().map(|v| v.0).collect::<Vec<_>>(),
            [Begin, terminal],
            "mode {mode}"
        );
    }
    clean(&screen);
}

#[test]
fn gradient_color_transforms_capture_and_child_edits_make_one_transaction() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let display_calls = Rc::new(RefCell::new(0));
    let calls = display_calls.clone();
    let (screen, (data, shown)) = kivact::test::render_with_handle(crate::options(), move || {
        let data = signal(stops());
        let selected = signal(Some(0));
        let shown = signal(true);
        let observer = callback(move |phase| seen.borrow_mut().push((phase, data.get_untracked())));
        (
            (data, shown),
            show(shown, move || {
                let calls = calls.clone();
                let factor = 0.5f32;
                let bias = 0.05f32;
                ui::gradient_editor(data, selected)
                    .color_transform(
                        move |color| {
                            *calls.borrow_mut() += 1;
                            let c = color.components();
                            Color::from_rgba_f32(c[0] * factor, c[1] * factor, c[2] * factor, c[3])
                        },
                        move |color| {
                            let c = color.components();
                            Color::from_rgba_f32(
                                c[0] / factor + bias,
                                c[1] / factor,
                                c[2] / factor,
                                c[3],
                            )
                        },
                    )
                    .on_edit(move |value| observer.try_call(value))
            }),
        )
    });
    let original = data.get_untracked();
    assert!(*display_calls.borrow() > 0);
    assert_eq!(original, stops());
    assert_eq!(data.get_untracked(), original);
    let red = screen.get_by_role(Role::SpinButton, "Color R");
    assert!(
        (red.value_range().2.unwrap() - f64::from(original[0].color.components()[0]) * 0.5).abs()
            < 0.001
    );
    red.a11y_with_value(A11yAction::SetValue, "0.2");
    assert!((data.get_untracked()[0].color.components()[0] - 0.45).abs() < 0.001);
    assert_eq!(log.borrow().len(), 2);
    assert_eq!(log.borrow()[0].0, Begin);
    assert_eq!(log.borrow()[1].0, Commit);
    let original = data.get_untracked();
    let p = red.rect().unwrap().center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 20.0, p.y);
    assert_ne!(data.get_untracked(), original);
    let moved = data.get_untracked();
    screen
        .get_by_role(Role::SpinButton, "Stop position")
        .a11y_with_value(A11yAction::SetValue, "0.9");
    assert_eq!(data.get_untracked(), moved);
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(data.get_untracked(), original);
    assert_eq!(log.borrow().last().map(|p| p.0), Some(Cancel));
    screen.advance(Duration::from_secs(1));
    let alpha = screen.get_by_role(Role::Slider, "Alpha").rect().unwrap();
    screen.mouse_move(alpha.center().x, alpha.center().y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(alpha.center().x + 20.0, alpha.center().y);
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(data.get_untracked(), original);
    assert_eq!(log.borrow().last().map(|p| p.0), Some(Cancel));
    clean(&screen);
}

#[test]
fn gradient_external_positions_preserve_storage_and_stops_remain_editable() {
    let (screen, data) = kivact::test::render_with_handle(crate::options(), || {
        let mut data = stops();
        data[0].position = -3.0;
        let data = signal(data);
        (data, ui::gradient_editor(data, signal(None)))
    });
    let original = data.get_untracked();
    let stop = screen.get_by_role(Role::Slider, "Gradient stop 0");
    assert_eq!(stop.value_range().2, Some(0.0));
    screen.screenshot();
    assert_eq!(data.get_untracked(), original);
    let p = stop.rect().unwrap().center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 30.0, p.y);
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(data.get_untracked(), original);
    clean(&screen);
    data.set(vec![
        ui::GradientStop::new(0.2, Color::BLACK),
        ui::GradientStop::new(0.8, Color::WHITE),
    ]);
    screen.run_until_idle();
    screen.get_by_role(Role::Slider, "Gradient stop 0");
    screen.get_by_role(Role::Slider, "Gradient stop 1");
    screen.get_by_role(Role::Button, "Add stop").click();
    assert_eq!(data.get_untracked().len(), 3);
    clean(&screen);
}

#[test]
fn gradient_deleting_before_selection_keeps_the_same_color_selected() {
    let (screen, (data, selected)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(stops());
        let selected = signal(Some(1_usize));
        ((data, selected), ui::gradient_editor(data, selected))
    });
    let color = data.get_untracked()[1].color;
    screen
        .get_by_role(Role::Slider, "Gradient stop 0")
        .press_key(Key::Delete);
    assert_eq!(selected.get_untracked(), Some(0));
    assert_eq!(data.get_untracked()[0].color, color);
    screen
        .get_by_role(Role::SpinButton, "Stop position")
        .a11y_with_value(A11yAction::SetValue, "0.6");
    assert_eq!(data.get_untracked()[0].position, 0.6);
    clean(&screen);
}
