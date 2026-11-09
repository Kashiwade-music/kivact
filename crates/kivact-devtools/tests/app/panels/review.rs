//! Role: Tests of the Review panel.

use std::path::PathBuf;

use kivact::prelude::*;

use crate::support;

#[test]
fn a_removed_node_is_marked_and_cannot_be_highlighted() {
    let fixture = support::fixture_with(vec![("get_review", review())]);
    let screen = support::open(&fixture, "Review");
    let present = screen.get_by_role(Role::ListItem, "C1 Button \"Save\"");
    let removed = screen.get_by_role(Role::ListItem, "C2 Button \"Gone\"");
    removed.get_by_text("(removed)");
    assert!(removed.get_by_role(Role::Button, "Highlight").is_disabled());
    present.get_by_role(Role::Button, "Highlight").click();
    screen.run_until_idle();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "highlight" && params == &serde_json::json!({"node": 2, "layout": false})
    }));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn adding_a_comment_uses_the_window_and_refetches() {
    let fixture = support::fixture_with(vec![
        ("get_review", review()),
        ("add_comment", added_item()),
    ]);
    let screen = support::open(&fixture, "Review");
    screen
        .get_by_role(Role::TextBox, "Comment")
        .type_text("Needs work");
    screen.get_by_role(Role::Button, "Add comment").click();
    screen.run_until_idle();
    let calls = fixture.calls.borrow();
    let added = calls
        .iter()
        .find(|(method, _)| method == "add_comment")
        .expect("invariant: the comment is sent");
    assert_eq!(
        added.1,
        serde_json::json!({"window": 1, "text": "Needs work", "screenshot": true})
    );
    assert!(
        calls
            .iter()
            .filter(|(method, _)| method == "get_review")
            .count()
            >= 2
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn open_folder_exports_the_review_and_reveals_the_json() {
    let fixture = support::fixture_with(vec![
        ("get_review", review()),
        (
            "export_review",
            serde_json::json!({
                "json": "C:/review/review.json",
                "markdown": "C:/review/review.md"
            }),
        ),
    ]);
    let screen = support::open(&fixture, "Review");
    screen.get_by_role(Role::Button, "Open folder").click();
    screen.run_until_idle();
    assert!(
        fixture
            .calls
            .borrow()
            .iter()
            .any(|(method, _)| method == "export_review")
    );
    assert_eq!(
        screen.revealed_paths(),
        vec![PathBuf::from("C:/review/review.json")]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A review with one live item and one item whose node is gone.
fn review() -> serde_json::Value {
    serde_json::json!({
        "schema": "kivact-devtools-review/1",
        "app": app(),
        "session": "test",
        "created": "2026-01-01T00:00:00Z",
        "updated": "2026-01-01T00:00:00Z",
        "environment": {"pseudo_locale": "off"},
        "items": [
            {
                "id": "C1",
                "window_title": "Main",
                "window": 1,
                "target": {
                    "kind": "node",
                    "node": 2,
                    "role": "button",
                    "name": "Save",
                    "bounds": {"x": 1.0, "y": 1.0, "width": 98.0, "height": 30.0}
                },
                "image": "images/C1.png",
                "comments": [
                    {"id": "C1.1", "text": "Make it bigger", "created": "2026-01-01T00:00:00Z"}
                ]
            },
            {
                "id": "C2",
                "window_title": "Main",
                "window": 1,
                "target": {
                    "kind": "node",
                    "node": 99,
                    "role": "button",
                    "name": "Gone",
                    "bounds": {"x": 0.0, "y": 0.0, "width": 10.0, "height": 10.0}
                },
                "comments": [
                    {"id": "C2.1", "text": "Old", "created": "2026-01-01T00:00:00Z"}
                ]
            }
        ]
    })
}

/// The item `add_comment` answers with.
fn added_item() -> serde_json::Value {
    serde_json::json!({
        "id": "C3",
        "window_title": "Main",
        "window": 1,
        "target": {"kind": "window"},
        "comments": [
            {"id": "C3.1", "text": "Needs work", "created": "2026-01-01T00:00:00Z"}
        ]
    })
}

/// The app of the review document.
fn app() -> serde_json::Value {
    serde_json::json!({
        "name": "Fixture",
        "pid": 42,
        "kivact_version": "0.1.0",
        "build": "debug",
        "exe": "target/debug/fixture",
        "cwd": "/work",
        "workspace_root": "/work"
    })
}
