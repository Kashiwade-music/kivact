//! Role: Tests of the DevTools app shell, shared by the panel tests.

use std::path::PathBuf;
use std::rc::Rc;

use kivact::prelude::*;
use kivact_devtools::state::Connection;
use kivact_devtools_protocol as p;

#[path = "app/support.rs"]
mod support;

#[path = "app/panels/accessibility.rs"]
mod accessibility;
#[path = "app/panels/animations.rs"]
mod animations;
#[path = "app/panels/console.rs"]
mod console;
#[path = "app/panels/elements.rs"]
mod elements;
#[path = "app/panels/emulation.rs"]
mod emulation;
#[path = "app/panels/input.rs"]
mod input;
#[path = "app/panels/performance.rs"]
mod performance;
#[path = "app/panels/review.rs"]
mod review;
#[path = "app/panels/scripts.rs"]
mod scripts;
#[path = "app/panels/tasks.rs"]
mod tasks;
#[path = "app/panels/theme.rs"]
mod theme;
#[path = "app/panels/updates.rs"]
mod updates;

#[test]
fn shell_shows_the_toolbar_and_every_tab() {
    let fixture = Rc::new(support::fixture());
    let screen = support::open(&fixture, "Elements");
    screen.get_by_role(Role::Button, "Pick");
    screen.get_by_role(Role::Button, "Record");
    for title in [
        "Elements",
        "Updates",
        "Performance",
        "Accessibility",
        "Input",
        "Animations",
        "Async",
        "Theme",
        "Console",
        "Emulation",
        "Review",
        "Scripts",
    ] {
        screen.get_by_role(Role::Tab, title);
    }
}

#[test]
fn lost_offers_reconnect_and_another_target() {
    let connection = Connection::Lost {
        path: PathBuf::from("target.json"),
        app_name: "Fixture".to_owned(),
        reason: "the connection was closed".to_owned(),
    };
    let screen = kivact::test::render(move || kivact_devtools::app::view_with(connection));
    screen.get_by_role(Role::Button, "Reconnect");
    screen.get_by_role(Role::Button, "Choose another target");
}

#[test]
fn choosing_lists_every_target() {
    let connection = Connection::Choosing(vec![target(1, "One"), target(2, "Two")]);
    let screen = kivact::test::render(move || kivact_devtools::app::view_with(connection));
    screen.get_by_role(Role::Button, "One · pid 1 · /workspace");
    screen.get_by_role(Role::Button, "Two · pid 2 · /workspace");
}

fn target(pid: u64, app_name: &str) -> (PathBuf, p::DiscoveryFile) {
    let file = serde_json::from_value(serde_json::json!({
        "protocol": p::PROTOCOL_VERSION,
        "pid": pid,
        "app_name": app_name,
        "kivact_version": "0.1.0",
        "port": 4000 + pid,
        "token": "token",
        "started_unix_ms": pid,
        "workspace_root": "/workspace"
    }))
    .expect("invariant: the test discovery file is valid");
    (PathBuf::from(format!("{pid}.json")), file)
}
