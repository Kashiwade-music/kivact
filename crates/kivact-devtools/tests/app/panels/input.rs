//! Role: Tests of the Input panel.

use std::rc::Rc;

use kivact::prelude::*;

use crate::support;

#[test]
fn switching_the_window_reloads_the_ime_state() {
    let mut fixture = support::fixture();
    fixture.windows.push(
        serde_json::from_value(serde_json::json!({
            "id": 2,
            "title": "Second",
            "bounds": {"x": 0.0, "y": 0.0, "width": 400.0, "height": 300.0},
            "scale": 1.0,
            "focused": false,
            "mode": "normal"
        }))
        .expect("invariant: the test window is valid"),
    );
    let fixture = Rc::new(fixture);
    let screen = support::open(&fixture, "Input");
    screen.get_by_role(Role::ComboBox, "Window").click();
    screen.get_by_role(Role::Option, "Second").click();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "get_ime_state" && params == &serde_json::json!({"window": 2})
    }));
}

#[test]
fn the_input_log_and_ime_state_are_shown() {
    let fixture = support::fixture_with(vec![
        (
            "list_inputs",
            serde_json::json!({
                "entries": [{
                    "seq": 1,
                    "at_ns": 1_000_000,
                    "window": 1,
                    "kind": "key",
                    "detail": "Enter",
                    "source": "platform",
                    "target": 2,
                    "route": [2, 1],
                    "handlers": [{
                        "node": 2,
                        "event": "key_down",
                        "outcome": "handled"
                    }]
                }]
            }),
        ),
        (
            "get_ime_state",
            serde_json::json!({
                "window": 1,
                "node": 2,
                "composing": true,
                "preedit": "ni",
                "purpose": "text"
            }),
        ),
    ]);
    let screen = support::open(&fixture, "Input");
    screen.get_by_text("key");
    screen.get_by_text("Enter");
    screen.get_by_text("key_down: handled");
    screen.get_by_text("Composing: true");
    screen.get_by_text("Preedit: ni");
}
