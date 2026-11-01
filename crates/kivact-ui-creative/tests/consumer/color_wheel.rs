//! Role: Color wheel model inputs, accessible targets and edit cancellation.
use super::*;
use kivact::style::Color;
use kivact::ui::EditPhase::{Begin, Cancel, Commit};
use kivact::view::{PointerKind, callback};

#[test]
fn color_wheel_regions_keys_actions_and_gray_hue_preservation() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, color) = kivact::test::render_with_handle(crate::options(), move || {
        let color = signal(Color::from_rgba_f32(0.5, 0.5, 0.5, 0.4));
        (
            color,
            ui::color_wheel(color, signal(ui::ColorModel::Rgb))
                .on_edit(move |phase| seen.borrow_mut().push(phase)),
        )
    });
    let hue = screen.get_by_role(Role::Slider, "Hue");
    for action in [
        A11yAction::Select,
        A11yAction::Activate,
        A11yAction::SetValue,
        A11yAction::Increment,
        A11yAction::Decrement,
    ] {
        assert!(hue.a11y_actions().contains(&action));
    }
    hue.a11y_with_value(A11yAction::SetValue, "120");
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    assert_eq!(color.get_untracked().components()[0], 0.5);
    let sv = screen.get_by_role(Role::Slider, "Saturation and value");
    sv.a11y_with_value(A11yAction::SetValue, "1,0.5");
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    let (h, s, v) = ui::color_to_hsv(color.get_untracked());
    assert!((h - 120.0).abs() < 0.01);
    assert!((s - 1.0).abs() < 0.01);
    assert!((v - 0.5).abs() < 0.01);
    hue.press_key(Key::ArrowRight);
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    screen.press_key_with(
        Key::ArrowUp,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    assert!((ui::color_to_hsv(color.get_untracked()).0 - 131.0).abs() < 0.01);
    let alpha = screen.get_by_role(Role::Slider, "Alpha");
    alpha.a11y_with_value(A11yAction::SetValue, "0.7");
    assert!((color.get_untracked().components()[3] - 0.7).abs() < 0.001);
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    sv.a11y_with_value(A11yAction::SetValue, "NaN,0.5");
    hue.a11y_with_value(A11yAction::SetValue, "inf");
    assert!(log.borrow().is_empty());
    clean(&screen);
}

#[test]
fn color_wheel_drag_final_point_and_all_cancel_paths_restore_before_notification() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (color, shown, disabled, hidden)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let color = signal(Color::from_rgba_f32(0.4, 0.6, 0.2, 0.8));
            let shown = signal(true);
            let disabled = signal(false);
            let hidden = signal(false);
            let observer =
                callback(move |phase| seen.borrow_mut().push((phase, color.get_untracked())));
            (
                (color, shown, disabled, hidden),
                show(shown, move || {
                    ui::color_wheel(color, signal(ui::ColorModel::Rgb))
                        .on_edit(move |value| observer.try_call(value))
                        .disabled(disabled)
                        .hidden(hidden)
                }),
            )
        });
    for mode in 0..6 {
        log.borrow_mut().clear();
        let original = color.get_untracked();
        let alpha = screen.get_by_role(Role::Slider, "Alpha");
        let r = alpha.rect().unwrap();
        let x = r.x + r.width * 0.25;
        let y = r.center().y;
        let contact = if mode == 2 {
            let contact = screen.contact_down(PointerKind::Touch, x, y);
            screen.contact_move(contact, x + 20.0, y);
            Some(contact)
        } else {
            screen.mouse_move(x, y);
            screen.mouse_down(MouseButton::Primary);
            screen.mouse_move(x + 20.0, y);
            None
        };
        assert_ne!(color.get_untracked(), original, "mode {mode}");
        let moved = color.get_untracked();
        screen
            .get_by_role(Role::Slider, "Hue")
            .a11y_with_value(A11yAction::SetValue, "250");
        assert_eq!(color.get_untracked(), moved);
        match mode {
            0 => {
                screen.mouse_move(r.x + r.width * 0.75, y);
                screen.mouse_up(MouseButton::Primary);
                assert!((color.get_untracked().components()[3] - 0.75).abs() < 0.02);
            }
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

        if mode > 0 {
            assert_eq!(color.get_untracked(), original);
        }
        assert_eq!(
            log.borrow().last().map(|v| v.1),
            Some(color.get_untracked())
        );
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
fn color_wheel_model_fields_hex_drafts_and_hidden_alpha() {
    let (screen, (color, model)) = kivact::test::render_with_handle(crate::options(), || {
        let color = signal(Color::from_rgba_f32(0.2, 0.3, 0.4, 0.6));
        let model = signal(ui::ColorModel::Rgb);
        (
            (color, model),
            ui::color_wheel(color, model)
                .alpha(false)
                .wheel_size(f32::NAN),
        )
    });
    assert!(screen.query_by_role(Role::Slider, "Alpha").is_none());
    assert!(
        screen
            .query_by_role(Role::SpinButton, "Color Alpha")
            .is_none()
    );
    screen
        .get_by_role(Role::SpinButton, "Color R")
        .a11y_with_value(A11yAction::SetValue, "0.7");
    assert!((color.get_untracked().components()[0] - 0.7).abs() < 0.001);
    model.set(ui::ColorModel::Hsv);
    screen.run_until_idle();
    screen
        .get_by_role(Role::SpinButton, "Color H")
        .a11y_with_value(A11yAction::SetValue, "180");
    assert!((ui::color_to_hsv(color.get_untracked()).0 - 180.0).abs() < 0.01);
    model.set(ui::ColorModel::Hex);
    screen.run_until_idle();
    let hex = screen.get_by_role(Role::TextBox, "Color HEX");
    hex.focus();
    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    hex.type_text("#ff0080");
    hex.press_key(Key::Enter);
    assert!((color.get_untracked().components()[3] - 0.6).abs() < 0.001);
    assert_eq!(color.get_untracked().components()[0], 1.0);
    let original = color.get_untracked();
    hex.focus();
    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    hex.type_text("#nonsense");
    hex.press_key(Key::Enter);
    assert_eq!(color.get_untracked(), original);
    assert!(hex.invalid());
    hex.press_key(Key::Escape);
    assert!(!hex.invalid());
    hex.focus();
    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    hex.type_text("#00Ff00aa");
    hex.press_key(Key::Enter);
    assert_eq!(color.get_untracked().components()[1], 1.0);
    assert!((color.get_untracked().components()[3] - 0.6).abs() < 0.001);
    clean(&screen);
}

#[test]
fn color_wheel_swatches_apply_add_remove_and_react() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (color, swatches, model)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let color = signal(rgb(0xff0000));
            let swatches = signal(vec![rgb(0x0000ff), rgb(0x00ff00)]);
            let model = signal(ui::ColorModel::Rgb);
            (
                (color, swatches, model),
                ui::color_wheel(color, model)
                    .swatches(swatches)
                    .on_edit(move |phase| seen.borrow_mut().push(phase)),
            )
        });
    screen.get_by_role(Role::ComboBox, "Color model").click();
    screen.get_by_role(Role::Option, "HSV").click();
    assert_eq!(model.get_untracked(), ui::ColorModel::Hsv);
    assert!(log.borrow().is_empty());
    screen.get_by_role(Role::Button, "Swatch 2").click();
    assert_eq!(color.get_untracked(), rgb(0x00ff00));
    screen.get_by_role(Role::Button, "Add swatch").click();
    assert_eq!(swatches.get_untracked().len(), 3);
    screen.get_by_role(Role::Button, "Remove swatch").click();
    assert_eq!(swatches.get_untracked().len(), 2);
    swatches.set(vec![Color::WHITE]);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Remove swatch").click();
    assert!(swatches.get_untracked().is_empty());
    assert_eq!(log.borrow().len(), 8);
    clean(&screen);
}

#[test]
fn color_wheel_pointer_hue_sv_and_release_coordinates() {
    let (screen, color) = kivact::test::render_with_handle(crate::options(), || {
        let color = signal(rgb(0xff0000));
        (
            color,
            ui::color_wheel(color, signal(ui::ColorModel::Rgb)).wheel_size(180.0),
        )
    });
    let hue = screen.get_by_role(Role::Slider, "Hue").rect().unwrap();
    screen.mouse_move(hue.center().x, hue.y + 2.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert!((ui::color_to_hsv(color.get_untracked()).0 - 270.0).abs() < 2.0);
    let sv = screen
        .get_by_role(Role::Slider, "Saturation and value")
        .rect()
        .unwrap();
    let contact = screen.contact_down(
        PointerKind::Touch,
        sv.x + sv.width * 0.2,
        sv.y + sv.height * 0.8,
    );
    screen.contact_move(contact, sv.x + sv.width * 0.8, sv.y + sv.height * 0.2);
    screen.contact_up(contact);
    let (_, s, v) = ui::color_to_hsv(color.get_untracked());
    assert!((s - 0.8).abs() < 0.02);
    assert!((v - 0.8).abs() < 0.02);
    clean(&screen);
}

#[test]
fn color_wheel_numeric_fields_share_one_scrub_transaction_and_hex_alpha() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (color, model)) = kivact::test::render_with_handle(crate::options(), move || {
        let color = signal(Color::from_rgba_f32(0.2, 0.4, 0.6, 0.8));
        let model = signal(ui::ColorModel::Rgb);
        let observer =
            callback(move |phase| seen.borrow_mut().push((phase, color.get_untracked())));
        (
            (color, model),
            ui::color_wheel(color, model).on_edit(move |value| observer.try_call(value)),
        )
    });
    let original = color.get_untracked();
    let red = screen.get_by_role(Role::SpinButton, "Color R");
    let p = red.rect().unwrap().center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 25.0, p.y);
    assert_ne!(color.get_untracked(), original);
    assert_eq!(log.borrow().len(), 1);
    assert_eq!(log.borrow()[0], (Begin, original));
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(color.get_untracked(), original);
    assert_eq!(log.borrow().last().copied(), Some((Cancel, original)));
    red.a11y_with_value(A11yAction::SetValue, "0.7");
    assert!((color.get_untracked().components()[0] - 0.7).abs() < 0.001);
    assert_eq!(log.borrow().len(), 4);
    model.set(ui::ColorModel::Hex);
    screen.run_until_idle();
    let hex = screen.get_by_role(Role::TextBox, "Color HEX");
    hex.focus();
    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    hex.type_text("#12345680");
    hex.press_key(Key::Enter);
    assert!((color.get_untracked().components()[3] - 128.0 / 255.0).abs() < 0.001);
    assert_eq!(log.borrow().len(), 6);
    clean(&screen);
}

#[test]
fn color_wheel_alpha_field_keeps_the_full_width_in_a_narrow_column() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            ui::color_wheel(signal(rgb(0x8b7cff)), signal(ui::ColorModel::Rgb))
                .wheel_size(200.0)
                .style(style().width(248)),
        )
    });
    let alpha = screen
        .get_by_role(Role::SpinButton, "Color Alpha")
        .rect()
        .expect("alpha field");
    assert!(alpha.width > 200.0, "{alpha:?}");
    let violations = screen.check_layout();
    assert!(violations.is_empty(), "{violations:?}");
    clean(&screen);
}

#[test]
fn color_wheel_can_hide_all_fields_while_wheel_alpha_and_swatches_keep_editing() {
    for model in [
        ui::ColorModel::Rgb,
        ui::ColorModel::Hsv,
        ui::ColorModel::Hex,
    ] {
        let (screen, color) = kivact::test::render_with_handle(crate::options(), move || {
            let color = signal(rgb(0x8b7cff));
            (
                color,
                ui::color_wheel(color, signal(model))
                    .fields(false)
                    .swatches(signal(vec![rgb(0xff0000)]))
                    .style(style().width(248)),
            )
        });
        assert!(
            screen
                .query_by_role(Role::ComboBox, "Color model")
                .is_none()
        );
        assert!(screen.query_by_role(Role::TextBox, "Color HEX").is_none());
        for name in [
            "Color R",
            "Color G",
            "Color B",
            "Color H",
            "Color S",
            "Color V",
            "Color Alpha",
        ] {
            assert!(screen.query_by_role(Role::SpinButton, name).is_none());
        }
        screen.get_by_role(Role::Slider, "Alpha");
        screen.get_by_role(Role::Button, "Swatch 1");
        screen
            .get_by_role(Role::Slider, "Hue")
            .a11y_with_value(A11yAction::SetValue, "120");
        assert!((ui::color_to_hsv(color.get_untracked()).0 - 120.0).abs() < 0.01);
        clean(&screen);
    }
}
