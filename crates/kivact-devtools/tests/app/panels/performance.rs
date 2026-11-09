//! Role: Tests of the Performance panel.

use std::rc::Rc;

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::support;

#[test]
fn export_trace_shows_the_written_path() {
    let fixture = support::fixture_with(vec![
        ("get_counters", counters()),
        (
            "export_trace",
            serde_json::json!({ "paths": ["/work/.kivact/traces/session-1.json"] }),
        ),
    ]);
    let screen = support::open(&fixture, "Performance");
    screen.get_by_role(Role::Canvas, "Frame times");
    screen.get_by_role(Role::Canvas, "Flame chart");
    screen.get_by_role(Role::Button, "Export trace").click();
    screen.get_by_text("/work/.kivact/traces/session-1.json");
    let calls = fixture.calls.borrow();
    assert!(
        calls.iter().any(|(method, params)| {
            method == "export_trace" && params == &serde_json::json!({})
        })
    );
}

#[test]
fn two_snapshots_are_differed_into_the_table() {
    let fixture = support::fixture_with(vec![
        ("get_counters", counters()),
        (
            "take_snapshot",
            serde_json::json!({ "id": 1, "taken_ns": 10, "owners": 1, "objects": 2 }),
        ),
        (
            "diff_snapshots",
            serde_json::json!({
                "added_objects": [{
                    "id": 7,
                    "owner": 1,
                    "name": "NewThing",
                    "source": {"file": "src/main.rs", "line": 3, "column": 7}
                }]
            }),
        ),
    ]);
    let screen = support::open(&fixture, "Performance");
    screen.mouse_move(400.0, 300.0);
    screen.scroll(0.0, 1000.0);
    let take = screen.get_by_role(Role::Button, "Take snapshot");
    take.click();
    take.click();
    screen.get_by_text("NewThing");
    let calls = fixture.calls.borrow();
    assert!(calls.iter().any(|(method, params)| {
        method == "diff_snapshots" && params == &serde_json::json!({ "before": 1, "after": 1 })
    }));
}

#[test]
fn a_recorded_frame_shows_its_renderer_and_budget() {
    let mut fixture = support::fixture();
    fixture
        .answers
        .push(("get_counters".to_owned(), counters()));
    let frame: p::FrameInfo = serde_json::from_value(serde_json::json!({
        "window": 1,
        "end_ns": 16_000_000,
        "work_ns": 20_000_000,
        "scene_ns": 2_000_000,
        "raster_ns": 3_000_000,
        "present_ns": 1_000_000,
        "chunks_rastered": 4,
        "pixels_rastered": 1000,
        "damage_pixels": 500,
        "renderer": "cpu",
        "frame_period_ns": 16_666_666,
        "over_budget": true
    }))
    .expect("invariant: the test frame is valid");
    fixture
        .notifications
        .push(p::Notification::FrameRecorded(frame));
    let fixture = Rc::new(fixture);
    let screen = support::open(&fixture, "Performance");
    screen.get_by_text("1 frame · 1 over budget · renderer: CPU (fallback)");
}

/// An answer of `get_counters`.
fn counters() -> serde_json::Value {
    serde_json::json!({
        "live_reactive": 4,
        "live_owners": 2,
        "live_objects": 3,
        "live_constants": 1,
        "tree_nodes": 3,
        "timers": 0,
        "frame_loops": 1,
        "animations": 0,
        "turns": 2,
        "outstanding_blocking": 0
    })
}
