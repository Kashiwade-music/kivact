//! Role: Tests of the Accessibility panel.

use std::rc::Rc;

use kivact::prelude::*;

use crate::support;

#[test]
fn switching_the_window_reloads_the_accessibility_tree() {
    let mut fixture = support::fixture();
    fixture.windows.push(second_window());
    let fixture = Rc::new(fixture);
    let screen = support::open(&fixture, "Accessibility");
    screen.get_by_role(Role::ComboBox, "Window").click();
    screen.get_by_role(Role::Option, "Second").click();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "get_a11y_tree" && params == &serde_json::json!({"window": 2})
    }));
}

fn second_window() -> kivact_devtools_protocol::WindowInfo {
    serde_json::from_value(serde_json::json!({
        "id": 2,
        "title": "Second",
        "bounds": {"x": 0.0, "y": 0.0, "width": 400.0, "height": 300.0},
        "scale": 1.0,
        "focused": false,
        "mode": "normal"
    }))
    .expect("invariant: the test window is valid")
}

#[test]
fn audit_rows_highlight_and_tab_order_toggles() {
    let fixture = support::fixture_with(vec![
        (
            "get_a11y_tree",
            serde_json::json!({
                "window": 1,
                "nodes": [{
                    "node": 2,
                    "semantics": {"role": "button", "name": "Save", "focusable": true},
                    "bounds": {"x": 1.0, "y": 2.0, "width": 30.0, "height": 20.0}
                }]
            }),
        ),
        (
            "audit_a11y",
            serde_json::json!({
                "window": 1,
                "findings": [{
                    "rule": "missing_name",
                    "node": 2,
                    "message": "Button has no name"
                }]
            }),
        ),
        (
            "get_focus",
            serde_json::json!({
                "window": 1,
                "node": 2,
                "visible": true,
                "history": [{
                    "at_ns": 1_000_000,
                    "before": 1,
                    "after": 2,
                    "reason": "keyboard"
                }]
            }),
        ),
        (
            "get_tab_order",
            serde_json::json!({"window": 1, "nodes": [2]}),
        ),
    ]);
    let screen = support::open(&fixture, "Accessibility");
    screen.get_by_text("Button has no name");
    screen.get_by_text("keyboard");

    screen.get_by_role(Role::Row, "Button has no name").click();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "highlight"
            && params["node"].as_u64() == Some(2)
            && params["layout"].as_bool() == Some(false)
    }));

    screen
        .get_by_role(Role::Switch, "Tab order numbers")
        .click();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "set_overlays" && params["tab_order"].as_bool() == Some(true)
    }));
}
