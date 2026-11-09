//! Role: Tests of the Theme panel.

use kivact::prelude::*;

use crate::support;

#[test]
fn theme_lists_tokens_with_previews_and_inputs() {
    let fixture = support::fixture_with(vec![(
        "list_theme_tokens",
        serde_json::json!({
            "tokens": [
                {"name": "Accent", "kind": "color", "value": "#2563eb"},
                {"name": "Radius", "kind": "length", "value": "6"}
            ]
        }),
    )]);
    let screen = support::open(&fixture, "Theme");
    screen.get_by_text("Accent");
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Accent")
            .value()
            .as_deref(),
        Some("#2563eb")
    );
    screen.get_by_role(Role::Slider, "Accent hue");
    screen.get_by_role(Role::Slider, "Accent saturation");
    screen.get_by_role(Role::Slider, "Accent lightness");
    screen.get_by_text("Radius");
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Radius")
            .value()
            .as_deref(),
        Some("6")
    );
}

#[test]
fn checking_a_token_sets_a_token_override() {
    let fixture = support::fixture_with(vec![
        (
            "list_theme_tokens",
            serde_json::json!({
                "tokens": [
                    {"name": "Accent", "kind": "color", "value": "#2563eb"}
                ]
            }),
        ),
        (
            "set_style_override",
            serde_json::json!({
                "id": 1,
                "scope": {"kind": "token", "token": "Accent"},
                "property": "Accent",
                "value": "#2563eb"
            }),
        ),
    ]);
    let screen = support::open(&fixture, "Theme");
    screen.get_by_role(Role::CheckBox, "Accent").click();
    let expected = serde_json::json!({
        "scope": {"kind": "token", "token": "Accent"},
        "property": "Accent",
        "value": "#2563eb"
    });
    let calls = fixture.calls.borrow();
    assert!(
        calls
            .iter()
            .any(|(method, params)| method == "set_style_override" && params == &expected),
        "expected set_style_override {expected}, got {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|(method, _)| method == "list_style_overrides"),
        "the override list is refetched after a set"
    );
}

#[test]
fn entering_a_token_value_sets_a_token_override() {
    let fixture = support::fixture_with(vec![
        (
            "list_theme_tokens",
            serde_json::json!({
                "tokens": [
                    {"name": "Radius", "kind": "length", "value": "6"}
                ]
            }),
        ),
        (
            "set_style_override",
            serde_json::json!({
                "id": 1,
                "scope": {"kind": "token", "token": "Radius"},
                "property": "Radius",
                "value": "8"
            }),
        ),
    ]);
    let screen = support::open(&fixture, "Theme");
    let input = screen.get_by_role(Role::TextBox, "Radius");
    input.press_key(Key::End);
    input.press_key(Key::Backspace);
    input.type_text("8");
    input.press_key(Key::Enter);
    let expected = serde_json::json!({
        "scope": {"kind": "token", "token": "Radius"},
        "property": "Radius",
        "value": "8"
    });
    let calls = fixture.calls.borrow();
    assert!(
        calls
            .iter()
            .any(|(method, params)| method == "set_style_override" && params == &expected),
        "expected set_style_override {expected}, got {calls:?}"
    );
}
