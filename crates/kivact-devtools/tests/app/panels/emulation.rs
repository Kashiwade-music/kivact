//! Role: Tests of the Emulation panel.

use kivact::prelude::*;

use crate::support;

#[test]
fn changing_the_theme_sends_the_whole_emulation() {
    let fixture = support::fixture_with(vec![
        ("get_emulation", serde_json::json!({"pseudo_locale": "off"})),
        (
            "set_emulation",
            serde_json::json!({"theme": "dark", "pseudo_locale": "off"}),
        ),
    ]);
    let screen = support::open(&fixture, "Emulation");
    screen.get_by_text(
        "Theme emulation only affects apps that choose their theme from the platform theme hint.",
    );
    screen.get_by_role(Role::ComboBox, "Theme").click();
    screen.get_by_role(Role::Option, "Dark").click();
    screen.run_until_idle();
    let calls = fixture.calls.borrow();
    assert!(calls.iter().any(|(method, params)| {
        method == "set_emulation"
            && params == &serde_json::json!({"theme": "dark", "pseudo_locale": "off"})
    }));
    assert!(
        calls
            .iter()
            .filter(|(method, _)| method == "get_emulation")
            .count()
            >= 2
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn switching_reduced_motion_off_sends_no_value_for_it() {
    let fixture = support::fixture_with(vec![
        (
            "get_emulation",
            serde_json::json!({"reduced_motion": true, "pseudo_locale": "off"}),
        ),
        ("set_emulation", serde_json::json!({"pseudo_locale": "off"})),
    ]);
    let screen = support::open(&fixture, "Emulation");
    screen.get_by_role(Role::Switch, "Reduced motion").click();
    screen.run_until_idle();
    let calls = fixture.calls.borrow();
    let sent: Vec<&serde_json::Value> = calls
        .iter()
        .filter(|(method, _)| method == "set_emulation")
        .map(|(_, params)| params)
        .collect();
    assert_eq!(sent, vec![&serde_json::json!({"pseudo_locale": "off"})]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_empty_dpi_scale_sends_no_scale() {
    let fixture = support::fixture_with(vec![
        (
            "get_emulation",
            serde_json::json!({"scale": 1.5, "pseudo_locale": "off"}),
        ),
        ("set_emulation", serde_json::json!({"pseudo_locale": "off"})),
    ]);
    let screen = support::open(&fixture, "Emulation");
    let input = screen.get_by_role(Role::TextBox, "DPI scale");
    input.press_key(Key::End);
    for _ in 0..3 {
        input.press_key(Key::Backspace);
    }
    input.press_key(Key::Enter);
    screen.run_until_idle();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "set_emulation" && params == &serde_json::json!({"pseudo_locale": "off"})
    }));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn reset_sends_an_emulation_with_no_setting() {
    let fixture = support::fixture_with(vec![
        (
            "get_emulation",
            serde_json::json!({
                "theme": "dark",
                "reduced_motion": true,
                "locale": "fr-CA",
                "pseudo_locale": "long"
            }),
        ),
        ("set_emulation", serde_json::json!({"pseudo_locale": "off"})),
    ]);
    let screen = support::open(&fixture, "Emulation");
    screen.get_by_role(Role::Button, "Reset").click();
    screen.run_until_idle();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "set_emulation" && params == &serde_json::json!({"pseudo_locale": "off"})
    }));
}

#[test]
fn toggling_reduced_motion_keeps_the_other_values() {
    let fixture = support::fixture_with(vec![
        (
            "get_emulation",
            serde_json::json!({"theme": "dark", "pseudo_locale": "long"}),
        ),
        (
            "set_emulation",
            serde_json::json!({"theme": "dark", "pseudo_locale": "long"}),
        ),
    ]);
    let screen = support::open(&fixture, "Emulation");
    screen.get_by_role(Role::Switch, "Reduced motion").click();
    screen.run_until_idle();
    let calls = fixture.calls.borrow();
    assert!(calls.iter().any(|(method, params)| {
        method == "set_emulation"
            && params
                == &serde_json::json!({
                    "theme": "dark",
                    "reduced_motion": true,
                    "pseudo_locale": "long"
                })
    }));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
