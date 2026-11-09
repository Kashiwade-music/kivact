//! Role: End-to-end contract of the DevTools client against a test App's agent.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with_handle};
use kivact_devtools_client::{ClientError, Connection, read_target};
use kivact_devtools_protocol as p;

/// What the client thread saw through one connection.
struct ClientReport {
    /// The turns recorded for the injected click.
    turns: p::TurnList,
    /// The screenshot path of the review item.
    image: Option<String>,
    /// The paths of the exported review files.
    exported: p::ExportedReview,
}

#[test]
fn a_devtools_client_inspects_drives_and_reviews_a_test_app() {
    let (screen, ()) = render_with_handle(RenderOptions::from_env().devtools(), || {
        let count = signal(0);
        (
            (),
            column((
                button("Increment")
                    .style(style().align_self(Align::Start))
                    .on_press(move || count.update(|c| *c += 1)),
                text(move || count.get().to_string()).test_id("count"),
            )),
        )
    });
    let before = screen
        .get_by_role(Role::Button, "Increment")
        .rect()
        .expect("the counter button is laid out");
    let discovery_path = screen
        .devtools_target()
        .expect("RenderOptions::devtools starts the agent");
    let discovery = read_target(&discovery_path).expect("the discovery file is readable");
    let done = Arc::new(AtomicBool::new(false));
    let finished = Arc::clone(&done);
    let (sender, receiver) = mpsc::channel();
    let client = std::thread::Builder::new()
        .name("kivact-devtools-e2e".to_owned())
        .spawn(move || {
            let _ = sender.send(run_client(&discovery));
            finished.store(true, Ordering::SeqCst);
        })
        .expect("the client thread starts");
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut grew = false;
    while !done.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline, "the client did not finish");
        screen.advance(Duration::from_millis(16));
        if let Some(rect) = screen.get_by_role(Role::Button, "Increment").rect() {
            grew |= rect.width > before.width;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    client.join().expect("the client thread ends");
    let report = receiver
        .recv()
        .expect("the client thread sends its report")
        .expect("the client calls succeed");
    assert!(grew, "the padding override widens the button");
    assert_eq!(screen.get_by_test_id("count").text(), "1");
    assert!(
        report.turns.turns.iter().any(|turn| {
            turn.nodes
                .iter()
                .any(|node| node.kind == p::CauseKind::PropChange)
        }),
        "the click's turn has a prop change"
    );
    assert!(report.image.is_some(), "the review item has a screenshot");
    assert!(Path::new(&report.exported.json).is_file());
    assert!(Path::new(&report.exported.markdown).is_file());
    drop(screen);
    let directory = Path::new(&report.exported.json)
        .parent()
        .expect("review.json is inside its session directory");
    std::fs::remove_dir_all(directory).expect("the exported review directory is removed");
}

/// Connect to `discovery`, inspect and override the button, record one click
/// and write a review. Runs on its own thread: the test thread drives the App.
fn run_client(discovery: &p::DiscoveryFile) -> Result<ClientReport, ClientError> {
    let (connection, _) = Connection::open(
        discovery,
        p::ClientInfo {
            name: "kivact-public-api-tests".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            kind: p::ClientKind::Other,
        },
    )?;
    let found = connection.call::<p::GetByRole>(p::RoleQuery {
        window: 1,
        within: None,
        role: p::Role::Button,
        name: Some("Increment".to_owned()),
        timeout_ms: None,
    })?;
    let node = found.node.id;
    connection.call::<p::Highlight>(p::HighlightParams {
        node: Some(node),
        layout: false,
    })?;
    connection.call::<p::SetStyleOverride>(p::SetStyleOverrideParams {
        scope: p::OverrideScope::Node { node },
        property: "padding".to_owned(),
        value: "20".to_owned(),
    })?;
    let latest = connection.call::<p::ListTurns>(p::ListTurnsParams {
        since: None,
        limit: Some(1),
        wait_ms: None,
    })?;
    let since = latest.turns.last().map(|turn| turn.turn);
    connection.call::<p::StartRecording>(p::RecordingSettings {
        updates: true,
        frames: false,
        input: false,
    })?;
    connection.call::<p::InjectInput>(p::InjectInputParams {
        window: 1,
        steps: vec![p::InputStep::Click { node }],
    })?;
    connection.call::<p::StopRecording>(p::Empty {})?;
    let turns = connection.call::<p::ListTurns>(p::ListTurnsParams {
        since,
        limit: None,
        wait_ms: None,
    })?;
    let item = connection.call::<p::AddComment>(p::AddCommentParams {
        window: 1,
        node: Some(node),
        text: "the counter button".to_owned(),
        screenshot: true,
    })?;
    let exported = connection.call::<p::ExportReview>(p::Empty {})?;
    Ok(ClientReport {
        turns,
        image: item.image,
        exported,
    })
}
