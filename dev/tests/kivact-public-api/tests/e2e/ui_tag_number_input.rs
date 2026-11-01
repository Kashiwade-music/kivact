//! Role: Tests the standard tag input and number input on real windows.
//!
//! The tag input's names, the keys that add and remove, its buttons and the
//! box that focuses its field; the number input's spin button semantics,
//! keys, buttons, typed text and the text it shows again when it loses the
//! focus.

use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::{EditPhase, ScalarRange, number_input, tag_input};
use kivact::view::{A11yAction, Modifiers};
use std::cell::RefCell;
use std::rc::Rc;

fn range(min: f64, max: f64, step: f64) -> ScalarRange {
    ScalarRange::new(min, max)
        .and_then(|range| range.with_step(step))
        .expect("invariant: finite bounds and step")
}

#[test]
fn a_tag_input_reports_trimmed_additions_and_removals_and_leaves_the_list_to_the_caller() {
    let (screen, (tags, added, removed)) = mount(|| {
        let tags = signal(vec!["rust".to_string(), "ui".to_string()]);
        let added = signal(Vec::<String>::new());
        let removed = signal(Vec::<usize>::new());
        (
            (tags, added, removed),
            tag_input(
                "Labels",
                tags,
                move |tag| added.update(|all| all.push(tag)),
                move |index| removed.update(|all| all.push(index)),
            )
            .test_id("tags"),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Labels");
    screen.get_by_role(Role::Button, "Remove rust");
    screen.get_by_role(Role::Button, "Remove ui");
    input.type_text("  go  ");
    input.press_key(Key::Enter);
    assert_eq!(added.get_untracked(), vec!["go"]);
    assert_eq!(input.value().as_deref(), Some(""), "the input is emptied");
    assert_eq!(tags.get_untracked().len(), 2, "the list is the caller's");
    input.press_key(Key::Backspace);
    assert_eq!(
        removed.get_untracked(),
        vec![1],
        "Backspace in an empty input"
    );
    input.type_text("a");
    input.press_key(Key::Backspace);
    assert_eq!(
        removed.get_untracked(),
        vec![1],
        "Backspace in a text edits the text"
    );
    screen.get_by_role(Role::Button, "Remove rust").click();
    assert_eq!(removed.get_untracked(), vec![1, 0]);

    input.press_key(Key::Backspace);
    input.type_text("   ");
    input.press_key(Key::Enter);
    assert_eq!(added.get_untracked().len(), 1, "a blank text adds nothing");

    tags.update(|all| all.push("new".to_string()));
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Remove new");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_press_on_the_box_focuses_the_input() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                button("Before"),
                tag_input("Labels", vec!["a".to_string()], |_| {}, |_| {}).test_id("tags"),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Before").focus();
    let rect = screen.get_by_test_id("tags").rect().expect("laid out");
    screen.click_at(rect.x + 3.0, rect.y + rect.height / 2.0);
    assert!(screen.get_by_role(Role::TextBox, "Labels").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_number_input_is_a_spin_button_with_keys_that_clamp_to_its_range() {
    let (screen, count) = mount(|| {
        let count = signal(4.0_f64);
        (count, number_input("Count", count, range(1.0, 12.0, 1.0)))
    });
    let field = screen.get_by_role(Role::SpinButton, "Count");
    assert_eq!(field.value().as_deref(), Some("4"));
    field.focus();
    screen.press_key(Key::ArrowLeft);
    assert_eq!(field.text_selection(), Some((0, 0)), "Left is the caret's");
    assert_eq!(count.get_untracked(), 4.0);
    for (key, expected) in [
        (Key::ArrowUp, 5.0),
        (Key::ArrowDown, 4.0),
        (Key::PageUp, 12.0),
        (Key::PageDown, 2.0),
        (Key::Home, 1.0),
        (Key::End, 12.0),
    ] {
        screen.press_key(key);
        assert_eq!(count.get_untracked(), expected);
        assert!(field.is_focused());
    }
    field.a11y_with_value(A11yAction::SetValue, "-100");
    assert_eq!(count.get_untracked(), 1.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_buttons_step_the_value_and_report_one_edit_each() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, count) = mount(move || {
        let count = signal(1.0_f64);
        (
            count,
            number_input("Count", count, range(0.0, 2.0, 1.0))
                .on_edit(move |phase| seen.borrow_mut().push(phase)),
        )
    });
    screen.get_by_role(Role::Button, "Increase").click();
    assert_eq!(count.get_untracked(), 2.0);
    screen.get_by_role(Role::Button, "Increase").click();
    assert_eq!(count.get_untracked(), 2.0, "the bound stops it");
    screen.get_by_role(Role::Button, "Decrease").click();
    assert_eq!(count.get_untracked(), 1.0);
    assert_eq!(
        log.borrow()[..2],
        [EditPhase::Begin, EditPhase::Commit],
        "a button press is one edit"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn typed_text_is_written_as_it_parses_and_shown_again_when_the_field_is_left() {
    let (screen, count) = mount(|| {
        let count = signal(1.0_f64);
        (
            count,
            column((
                number_input("Amount", count, range(0.0, 10.0, 0.25)),
                button("Elsewhere"),
            )),
        )
    });
    let field = screen.get_by_role(Role::SpinButton, "Amount");
    assert_eq!(field.value().as_deref(), Some("1"));
    field.focus();
    field.press_key(Key::End);
    field.type_text(".5");
    assert_eq!(count.get_untracked(), 1.5);
    field.type_text("x");
    assert_eq!(
        count.get_untracked(),
        1.5,
        "a text that does not parse writes nothing"
    );
    field.press_key(Key::Backspace);
    field.press_key(Key::ArrowLeft);
    field.press_key(Key::ArrowLeft);
    field.press_key(Key::ArrowLeft);
    field.type_text("9");
    assert_eq!(count.get_untracked(), 10.0, "clamped to the range");
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    screen.run_until_idle();
    assert_eq!(field.value().as_deref(), Some("10"));
    count.set(2.75);
    screen.run_until_idle();
    assert_eq!(field.value().as_deref(), Some("2.75"), "follows the value");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disabled_composite_inputs_disable_every_editing_control_and_follow_the_source() {
    let (screen, (disabled, count, added, removed)) = mount(|| {
        let disabled = signal(true);
        let count = signal(1.0);
        let added = signal(0);
        let removed = signal(0);
        (
            (disabled, count, added, removed),
            column((
                tag_input(
                    "Labels",
                    vec!["rust".to_string()],
                    move |_| added.update(|n| *n += 1),
                    move |_| removed.update(|n| *n += 1),
                )
                .disabled(disabled),
                number_input("Count", count, range(0.0, 10.0, 1.0)).disabled(disabled),
            )),
        )
    });
    for (role, name) in [
        (Role::TextBox, "Labels"),
        (Role::Button, "Remove rust"),
        (Role::SpinButton, "Count"),
        (Role::Button, "Increase"),
        (Role::Button, "Decrease"),
    ] {
        assert!(
            screen.get_by_role(role, name).is_disabled(),
            "{name} inherits disabled"
        );
    }
    screen.get_by_role(Role::Button, "Increase").force_click();
    screen
        .get_by_role(Role::Button, "Remove rust")
        .force_click();
    screen
        .get_by_role(Role::SpinButton, "Count")
        .a11y_with_value(A11yAction::SetValue, "8");
    assert_eq!(count.get_untracked(), 1.0);
    assert_eq!((added.get_untracked(), removed.get_untracked()), (0, 0));
    disabled.set(false);
    screen.run_until_idle();
    let labels = screen.get_by_role(Role::TextBox, "Labels");
    assert!(!labels.is_disabled());
    labels.type_text("go");
    labels.press_key(Key::Enter);
    screen.get_by_role(Role::Button, "Remove rust").click();
    screen.get_by_role(Role::Button, "Increase").click();
    assert_eq!(count.get_untracked(), 2.0);
    assert_eq!((added.get_untracked(), removed.get_untracked()), (1, 1));
    disabled.set(true);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::SpinButton, "Count").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn number_input_preserves_tiny_steps_and_unstepped_external_values() {
    let (screen, (tiny, precise)) = mount(|| {
        let tiny = signal(0.0000001);
        let precise = signal(1.2345678901234567);
        (
            (tiny, precise),
            column((
                number_input("Tiny", tiny, range(0.0, 1.0, 0.0000001)),
                number_input(
                    "Precise",
                    precise,
                    ScalarRange::new(0.0, 10.0).expect("finite range"),
                ),
            )),
        )
    });
    let tiny_field = screen.get_by_role(Role::SpinButton, "Tiny");
    let precise_field = screen.get_by_role(Role::SpinButton, "Precise");
    assert_eq!(tiny_field.value().as_deref(), Some("0.0000001"));
    assert_eq!(
        precise_field.value().unwrap().parse::<f64>().unwrap(),
        precise.get_untracked()
    );
    tiny.set(0.0000003);
    precise.set(2.345678901234567);
    screen.run_until_idle();
    assert_eq!(tiny_field.value().as_deref(), Some("0.0000003"));
    assert_eq!(
        precise_field.value().unwrap().parse::<f64>().unwrap(),
        precise.get_untracked()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn typed_text_stays_while_the_field_has_the_focus_even_when_clamped_or_rounded() {
    let (screen, count) = mount(|| {
        let count = signal(5.0_f64);
        (
            count,
            column((
                number_input("Amount", count, range(0.0, 100.0, 1.0)),
                button("Elsewhere"),
            )),
        )
    });
    let field = screen.get_by_role(Role::SpinButton, "Amount");
    field.focus();
    field.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    field.type_text("150");
    assert_eq!(count.get_untracked(), 100.0, "clamped to the range");
    assert_eq!(
        field.value().as_deref(),
        Some("150"),
        "the typed text stays while the field has the focus"
    );
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    screen.run_until_idle();
    assert_eq!(field.value().as_deref(), Some("100"));
    field.focus();
    field.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    field.type_text("50");
    assert_eq!(count.get_untracked(), 50.0);
    assert_eq!(field.value().as_deref(), Some("50"));
    field.press_key(Key::ArrowUp);
    assert_eq!(count.get_untracked(), 51.0);
    assert_eq!(
        field.value().as_deref(),
        Some("51"),
        "a key step replaces the typed text"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn shown_precision_follows_the_step_grid_that_starts_at_the_minimum_and_the_maximum() {
    let (screen, _) = mount(|| {
        let low = signal(0.15_f64);
        let high = signal(1.05_f64);
        (
            (low, high),
            column((
                number_input("Low", low, range(0.05, 1.0, 0.1)),
                number_input("High", high, range(0.0, 1.05, 0.1)),
            )),
        )
    });
    assert_eq!(
        screen
            .get_by_role(Role::SpinButton, "Low")
            .value()
            .as_deref(),
        Some("0.15")
    );
    assert_eq!(
        screen
            .get_by_role(Role::SpinButton, "High")
            .value()
            .as_deref(),
        Some("1.05")
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
