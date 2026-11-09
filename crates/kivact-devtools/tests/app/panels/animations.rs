//! Role: Tests of the Animations panel.

use kivact::prelude::*;

use crate::support;

#[test]
fn animations_are_listed_and_the_time_scale_is_sent() {
    let fixture = support::fixture_with(vec![(
        "list_animations",
        serde_json::json!({
            "animations": [{
                "id": 7,
                "kind": "timeline",
                "owner": 3,
                "running": true,
                "paused": false,
                "decorative": false
            }],
            "time_scale": 1.0
        }),
    )]);

    let screen = support::open(&fixture, "Animations");
    screen.get_by_role(Role::GridCell, "timeline");
    screen.get_by_role(Role::GridCell, "7");

    screen.get_by_role(Role::Button, "0.5x").click();
    assert!(fixture.calls.borrow().iter().any(|(method, params)| {
        method == "set_time_scale" && params["scale"].as_f64() == Some(0.5)
    }));
}
