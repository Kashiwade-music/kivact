//! Role: Tests of the Updates panel.

use std::rc::Rc;

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::support;

#[test]
fn selecting_a_turn_shows_its_header_and_a_card_highlights_its_node() {
    let mut fixture = support::fixture();
    fixture.notifications = vec![p::Notification::TurnRecorded(turn())];
    let fixture = Rc::new(fixture);
    let screen = support::open(&fixture, "Updates");
    screen.get_by_role(Role::Button, "Turn 1").click();
    screen.get_by_text("#1 click   total 5.0 ms");
    screen.get_by_role(Role::Button, "i32 ×3").click();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "highlight" && params == &serde_json::json!({"node": 2, "layout": false})
    }));
}

#[test]
fn paint_flashing_toggles_the_overlay() {
    let fixture = support::fixture_with(Vec::new());
    let screen = support::open(&fixture, "Updates");
    screen.get_by_role(Role::Button, "Paint flashing").click();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "set_overlays"
            && params.get("paint_flashing") == Some(&serde_json::Value::Bool(true))
    }));
}

fn turn() -> p::TurnGraph {
    serde_json::from_value(serde_json::json!({
        "turn": 1,
        "started_ns": 1_000_000,
        "duration_ns": 5_000_000,
        "stages": [
            {"stage": "begin", "start_ns": 1_000_000, "duration_ns": 1_000_000},
            {"stage": "dispatch", "start_ns": 2_000_000, "duration_ns": 4_000_000}
        ],
        "nodes": [
            {"id": 0, "kind": "input", "label": "click", "count": 1, "wasted": false},
            {"id": 1, "kind": "signal_write", "label": "i32", "count": 3,
             "node": 2, "wasted": false}
        ],
        "edges": [{"source": 0, "target": 1}],
        "changed_nodes": [2],
        "wasted": {"equal_writes": 0, "equal_memos": 0, "unchanged_props": 0},
        "truncated": false
    }))
    .expect("invariant: the test turn is valid")
}
