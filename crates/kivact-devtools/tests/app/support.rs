//! Role: Shared fixture data and rendering helpers for the DevTools app tests.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::prelude::*;
use kivact::test::Screen;
use kivact_devtools::source::{Fixture, Source};
use kivact_devtools::state::{Connection, Target};
use kivact_devtools_protocol as p;

/// Render the app on `fixture` and open the tab `panel`.
pub(crate) fn open(fixture: &Rc<Fixture>, panel: &str) -> Screen {
    let connection = connected(fixture);
    let screen = kivact::test::render(move || kivact_devtools::app::view_with(connection));
    screen.get_by_role(Role::Tab, panel).click();
    screen
}

/// One window "Main" (id 1), a tree, the detail of node 2, one turn and two frames.
pub(crate) fn fixture() -> Fixture {
    Fixture {
        app: app(),
        windows: windows(),
        trees: vec![tree()],
        nodes: vec![detail()],
        turns: vec![turn()],
        frames: frames(),
        answers: vec![(
            "get_overlays".to_owned(),
            serde_json::json!({
                "paint_flashing": false,
                "layout_lint": false,
                "update_heatmap": false,
                "tab_order": false,
                "selected_only": false
            }),
        )],
        notifications: Vec::new(),
        calls: RefCell::new(Vec::new()),
    }
}

/// `fixture()` with extra `answers` (method name and result JSON).
pub(crate) fn fixture_with(answers: Vec<(&str, serde_json::Value)>) -> Rc<Fixture> {
    let mut fixture = fixture();
    fixture.answers.extend(
        answers
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value)),
    );
    Rc::new(fixture)
}

/// A connected target that answers from `fixture`.
pub(crate) fn connected(fixture: &Rc<Fixture>) -> Connection {
    let config = serde_json::from_value(serde_json::json!({}))
        .expect("invariant: all configuration fields default");
    Connection::Connected(Target {
        source: Source::Fixture(fixture.clone()),
        app: fixture.app.clone(),
        windows: fixture.windows.clone(),
        config: Rc::new(config),
        path: None,
    })
}

fn app() -> p::AppInfo {
    serde_json::from_value(serde_json::json!({
        "name": "Fixture",
        "pid": 42,
        "kivact_version": "0.1.0",
        "build": "debug",
        "exe": "target/debug/fixture",
        "cwd": "/work",
        "workspace_root": "/work"
    }))
    .expect("invariant: the test app is valid")
}

fn windows() -> Vec<p::WindowInfo> {
    vec![window()]
}

fn window() -> p::WindowInfo {
    serde_json::from_value(serde_json::json!({
        "id": 1,
        "title": "Main",
        "bounds": {"x": 0.0, "y": 0.0, "width": 800.0, "height": 600.0},
        "scale": 1.0,
        "focused": true,
        "mode": "normal"
    }))
    .expect("invariant: the test window is valid")
}

fn tree() -> p::TreeSnapshot {
    p::TreeSnapshot {
        window: 1,
        root: 1,
        revision: 1,
        nodes: vec![root(), button(), text_node()],
    }
}

fn root() -> p::ElementInfo {
    serde_json::from_value(serde_json::json!({
        "id": 1,
        "window": 1,
        "children": [2, 3],
        "kind": "root",
        "hidden": false,
        "leaving": false,
        "update_count": 0
    }))
    .expect("invariant: the test root is valid")
}

fn button() -> p::ElementInfo {
    serde_json::from_value(serde_json::json!({
        "id": 2,
        "window": 1,
        "parent": 1,
        "kind": "box",
        "role": "button",
        "name": "Save",
        "test_id": "save",
        "source": {"file": "src/main.rs", "line": 10, "column": 5},
        "hidden": false,
        "leaving": false,
        "update_count": 3
    }))
    .expect("invariant: the test button is valid")
}

fn text_node() -> p::ElementInfo {
    serde_json::from_value(serde_json::json!({
        "id": 3,
        "window": 1,
        "parent": 1,
        "kind": "text",
        "text": "Hello",
        "hidden": false,
        "leaving": false,
        "update_count": 0
    }))
    .expect("invariant: the test text is valid")
}

fn detail() -> p::NodeDetail {
    serde_json::from_value(serde_json::json!({
        "node": button(),
        "bounds": {
            "margin": {"x": 0.0, "y": 0.0, "width": 100.0, "height": 32.0},
            "border": {"x": 1.0, "y": 1.0, "width": 98.0, "height": 30.0},
            "padding": {"x": 5.0, "y": 5.0, "width": 90.0, "height": 22.0},
            "content": {"x": 10.0, "y": 10.0, "width": 80.0, "height": 12.0}
        },
        "semantics": {"role": "button", "name": "Save", "focusable": true}
    }))
    .expect("invariant: the test node detail is valid")
}

fn turn() -> p::TurnGraph {
    serde_json::from_value(serde_json::json!({
        "turn": 1,
        "started_ns": 0,
        "duration_ns": 5_000_000,
        "changed_nodes": [2],
        "wasted": {"equal_writes": 0, "equal_memos": 0, "unchanged_props": 0},
        "truncated": false
    }))
    .expect("invariant: the test turn is valid")
}

fn frames() -> Vec<p::FrameInfo> {
    vec![frame(16_000_000), frame(33_000_000)]
}

fn frame(end_ns: u64) -> p::FrameInfo {
    serde_json::from_value(serde_json::json!({
        "window": 1,
        "end_ns": end_ns,
        "work_ns": 8_000_000,
        "scene_ns": 2_000_000,
        "raster_ns": 3_000_000,
        "present_ns": 1_000_000,
        "chunks_rastered": 4,
        "pixels_rastered": 1000,
        "damage_pixels": 500,
        "renderer": "cpu",
        "frame_period_ns": 16_666_666,
        "over_budget": false
    }))
    .expect("invariant: the test frame is valid")
}
