//! Role: Test Numeric's value text, mapping, steps and scrub policy.

use kivact::interaction::{InteractionPolicy, ModifierKey, ScalarMapping};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen};
use kivact::view::EditPhase;
use kivact::view::KeyCode;
use kivact::view::Modifiers;
use kivact::view::MouseButton;
use kivact::view::Role;
use kivact_ui_creative as ui;
use std::cell::RefCell;
use std::rc::Rc;

fn center(screen: &Screen, role: Role, name: &str) -> (f64, f64) {
    let rect = screen.get_by_role(role, name).rect().expect("laid out");
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

fn hex_to_gain(db: f64) -> f64 {
    10f64.powf(db / 20.0)
}

/// A theme whose scrub policy is `edit` on top of the kit's theme.
fn themed(edit: impl FnOnce(&mut InteractionPolicy)) -> RenderOptions {
    let mut policy =
        (*ui::theme(kivact::style::ThemeMode::Dark).ext::<InteractionPolicy>()).clone();
    edit(&mut policy);
    RenderOptions::with_theme(ui::theme(kivact::style::ThemeMode::Dark).with(policy))
        .reduced_motion()
}

#[test]
fn a_formatter_sets_the_visible_and_accessible_text() {
    let screen = kivact::test::render_with(crate::support::options(), || {
        ui::number("Gain", signal(0.5))
            .range(0.0..=2.0)
            .format(|gain| format!("{:.1} dB", 20.0 * gain.log10()))
    });
    screen.get_by_text("-6.0 dB");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_text_field_opens_with_the_formatted_text_and_reads_it_with_the_parser() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(0.5);
        (
            value,
            ui::number("Gain", value)
                .range(0.0..=2.0)
                .format(|gain| format!("{:.1} dB", 20.0 * gain.log10()))
                .text_parser(|text, _| {
                    let db: f64 = text
                        .trim()
                        .trim_end_matches("dB")
                        .trim()
                        .parse()
                        .map_err(|_| "Enter decibels".to_owned())?;
                    Ok(hex_to_gain(db))
                }),
        )
    });
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .press_key(Key::Enter);
    let field = screen.get_by_role(Role::TextBox, "Gain");
    assert_eq!(field.value().as_deref(), Some("-6.0 dB"));
    field.type_text("-12 dB");
    screen.press_key(Key::Enter);
    assert!((value.get_untracked() - hex_to_gain(-12.0)).abs() < 0.001);
}

#[test]
fn assistive_set_value_reads_text_with_the_parser() {
    let phases = Rc::new(RefCell::new(Vec::new()));
    let seen = phases.clone();
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(0.5);
        (
            value,
            ui::number("Gain", value)
                .range(0.0..=2.0)
                .format(|gain| format!("{:.1} dB", 20.0 * gain.log10()))
                .text_parser(|text, _| {
                    let db: f64 = text
                        .trim()
                        .trim_end_matches("dB")
                        .trim()
                        .parse()
                        .map_err(|_| "Enter decibels".to_owned())?;
                    Ok(hex_to_gain(db))
                })
                .on_edit(move |phase| seen.borrow_mut().push(phase)),
        )
    });
    let gain = screen.get_by_role(Role::SpinButton, "Gain");
    gain.a11y_set_value("-12 dB");
    assert!((value.get_untracked() - hex_to_gain(-12.0)).abs() < 0.001);
    assert_eq!(*phases.borrow(), [EditPhase::Begin, EditPhase::Commit]);
    phases.borrow_mut().clear();
    gain.a11y_set_value("abc");
    assert!((value.get_untracked() - hex_to_gain(-12.0)).abs() < 0.001);
    assert!(phases.borrow().is_empty(), "unparsable text writes nothing");
}

#[test]
fn range_step_quantizes_numeric_writes() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(0.0);
        (
            value,
            ui::number("Gain", value)
                .range(
                    kivact::interaction::ScalarRange::new(0.0, 1.0)
                        .expect("valid range")
                        .with_step(0.25)
                        .expect("valid step"),
                )
                .step(0.01),
        )
    });
    let (x, y) = center(&screen, Role::SpinButton, "Gain");
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 10.0, y);
    screen.mouse_up(MouseButton::Primary);
    let written = value.get_untracked();
    assert!(
        (written / 0.25 - (written / 0.25).round()).abs() < 1e-9,
        "{written}"
    );
}

#[test]
fn drag_distance_sets_the_full_range_travel() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(0.0);
        (
            value,
            ui::knob("Gain", value)
                .range(0.0..=1.0)
                .drag_distance(200.0),
        )
    });
    let (x, y) = center(&screen, Role::Slider, "Gain");
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x, y - 100.0);
    screen.mouse_up(MouseButton::Primary);
    assert!((value.get_untracked() - 0.5).abs() < 0.001);
}

#[test]
fn page_keys_move_ten_steps() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(0.0);
        (
            value,
            ui::number("Level", value).range(0.0..=100.0).step(1.0),
        )
    });
    let level = screen.get_by_role(Role::SpinButton, "Level");
    level.press_key(Key::PageUp);
    assert_eq!(value.get_untracked(), 10.0);
    level.press_key(Key::PageDown);
    assert_eq!(value.get_untracked(), 0.0);
}

#[test]
fn small_wheel_deltas_add_up_to_one_step() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(0.0);
        (
            value,
            ui::number("Level", value).range(-10.0..=10.0).step(1.0),
        )
    });
    let level = screen.get_by_role(Role::SpinButton, "Level");
    level.scroll(0.0, 20.0);
    level.scroll(0.0, 20.0);
    assert_eq!(value.get_untracked(), 0.0, "sub-notch events write nothing");
    level.scroll(0.0, 20.0);
    assert_eq!(value.get_untracked(), -1.0);
    level.scroll(0.0, -60.0);
    assert_eq!(value.get_untracked(), 0.0, "a reversed scroll starts over");
    level.scroll(30.0, 0.0);
    level.scroll(30.0, 0.0);
    assert_eq!(value.get_untracked(), -1.0, "a sideways wheel decreases");
}

#[test]
fn a_reset_click_resets_like_a_double_click() {
    let (screen, value) = kivact::test::render_with_handle(
        themed(|policy| policy.scrub.reset_click = Some(ModifierKey::Alt)),
        move || {
            let value = signal(3.0);
            (
                value,
                ui::number("Level", value).range(0.0..=10.0).reset_to(7.0),
            )
        },
    );
    screen.key_down(
        Key::Unidentified,
        KeyCode::AltLeft,
        Modifiers {
            alt: true,
            ..Modifiers::default()
        },
    );
    screen.get_by_role(Role::SpinButton, "Level").click();
    screen.key_up(Key::Unidentified, KeyCode::AltLeft, Modifiers::default());
    assert_eq!(value.get_untracked(), 7.0);
}

#[test]
fn a_knob_opens_its_text_field_with_enter() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(50.0);
        (
            value,
            ui::knob("Mix", value).range(0.0..=100.0).text_entry(true),
        )
    });
    let knob = screen.get_by_role(Role::Slider, "Mix");
    knob.press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Mix").type_text("25");
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 25.0);
    knob.press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Mix").type_text("80");
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 25.0);
    screen.assert_layout_clean();
}

#[test]
fn a_knob_without_text_entry_ignores_enter() {
    let screen = kivact::test::render_with(crate::support::options(), || {
        ui::knob("Mix", signal(50.0)).range(0.0..=100.0)
    });
    screen
        .get_by_role(Role::Slider, "Mix")
        .press_key(Key::Enter);
    assert!(screen.query_by_role(Role::TextBox, "Mix").is_none());
}

#[test]
fn a_logarithmic_knob_scrubs_through_the_mapping() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(20.0);
        (
            value,
            ui::knob("Frequency", value)
                .range(20.0..=20_000.0)
                .mapping(ScalarMapping::logarithmic()),
        )
    });
    let rect = screen
        .get_by_role(Role::Slider, "Frequency")
        .rect()
        .expect("laid out");
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x, y - rect.height / 2.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(
        (value.get_untracked() - 632.4555).abs() < 30.0,
        "{}",
        value.get_untracked()
    );
}

#[test]
fn a_fader_defaults_to_the_square_root_curve() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(0.0);
        (value, ui::fader("Volume", value).range(0.0..=1.0))
    });
    let rect = screen
        .get_by_role(Role::Slider, "Volume")
        .rect()
        .expect("laid out");
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x, y - rect.height / 2.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(
        (value.get_untracked() - 0.25).abs() < 0.01,
        "{}",
        value.get_untracked()
    );
}

#[test]
fn a_fine_key_held_mid_scrub_does_not_jump() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(0.0);
        (value, ui::knob("Gain", value).range(0.0..=1.0))
    });
    let rect = screen
        .get_by_role(Role::Slider, "Gain")
        .rect()
        .expect("laid out");
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.get_by_role(Role::Slider, "Gain").focus();
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x, y - 50.0);
    let before = value.get_untracked();
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    screen.mouse_move(x, y - 60.0);
    let after = value.get_untracked();
    let extent = rect.height.max(60.0);
    assert!(
        (after - before - 10.0 * 0.1 / extent).abs() < 0.005,
        "{before} -> {after}"
    );
    screen.mouse_up(MouseButton::Primary);
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
}

#[test]
fn the_scrub_policy_chooses_the_fine_key() {
    let (screen, value) = kivact::test::render_with_handle(
        themed(|policy| {
            policy.scrub.fine = Some(ModifierKey::Shift);
            policy.scrub.coarse = None;
        }),
        move || {
            let value = signal(0.0);
            (
                value,
                ui::number("Level", value).range(0.0..=10.0).step(1.0),
            )
        },
    );
    let level = screen.get_by_role(Role::SpinButton, "Level");
    level.press_key_with(
        Key::ArrowUp,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert!((value.get_untracked() - 0.1).abs() < 1e-9, "Shift is fine");
    level.press_key_with(
        Key::ArrowUp,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    assert!(
        (value.get_untracked() - 1.1).abs() < 1e-9,
        "Ctrl is not coarse"
    );
}

#[test]
fn a_mixed_value_does_not_call_the_formatter() {
    let calls = Rc::new(RefCell::new(0_usize));
    let seen = calls.clone();
    let screen = kivact::test::render_with(crate::support::options(), move || {
        ui::number("Gain", signal(0.5))
            .range(0.0..=1.0)
            .mixed(true)
            .format(move |value| {
                *seen.borrow_mut() += 1;
                format!("{value:.2}")
            })
    });
    screen.get_by_text("Mixed");
    assert_eq!(*calls.borrow(), 0);
}
