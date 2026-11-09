//! Role: Tests of the Input panel.

use crate::support;

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
