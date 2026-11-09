//! Role: Tests of the Async panel.

use kivact::prelude::*;

use crate::support;

#[test]
fn tasks_resources_and_the_blocking_count_are_shown() {
    let fixture = support::fixture_with(vec![(
        "list_tasks",
        serde_json::json!({
            "tasks": [{
                "id": 5,
                "kind": "resource",
                "phase": "pending",
                "owner": 2,
                "deadline_ns": 10_000_000
            }],
            "outstanding_blocking": 2
        }),
    )]);

    let screen = support::open(&fixture, "Async");
    screen.get_by_text("Outstanding blocking: 2");
    screen.get_by_role(Role::GridCell, "resource");
    screen.get_by_role(Role::GridCell, "pending");
    screen.get_by_role(Role::GridCell, "10.000 ms");
}
