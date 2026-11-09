//! Role: End-to-end contract of the DevTools client against a test App's agent.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with_handle};
use kivact_devtools_client::{Connection, read_target};
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
    // The client keeps its connection (and so the target's overrides) alive
    // until the test thread has checked the App against them.
    let (release, released) = mpsc::channel();
    let client = std::thread::Builder::new()
        .name("kivact-devtools-e2e".to_owned())
        .spawn(move || match run_client(&discovery) {
            Ok((connection, report)) => {
                let _ = sender.send(Ok(report));
                finished.store(true, Ordering::SeqCst);
                let _ = released.recv();
                drop(connection);
            }
            Err(error) => {
                let _ = sender.send(Err(error));
                finished.store(true, Ordering::SeqCst);
            }
        })
        .expect("the client thread starts");
    let deadline = Instant::now() + Duration::from_secs(60);
    while !done.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline, "the client did not finish");
        screen.advance(Duration::from_millis(16));
        std::thread::sleep(Duration::from_millis(1));
    }
    let report = receiver
        .recv()
        .expect("the client thread sends its report")
        .expect("the client calls succeed");
    let after = screen
        .get_by_role(Role::Button, "Increment")
        .rect()
        .expect("the counter button is laid out");
    let _ = release.send(());
    client.join().expect("the client thread ends");
    assert!(
        after.width > before.width,
        "the padding override widens the button: before {before:?}, after {after:?}"
    );
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

/// Connect to `discovery`, inspect and drive the button and write a review.
/// Runs on its own thread: the test thread drives the App.
fn run_client(discovery: &p::DiscoveryFile) -> Result<(Connection, ClientReport), String> {
    let (connection, _) = Connection::open(
        discovery,
        p::ClientInfo {
            name: "kivact-public-api-tests".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            kind: p::ClientKind::Other,
        },
    )
    .map_err(|error| format!("initialize: {error}"))?;
    let (node, debug) = inspect_button(&connection)?;
    let turns = override_and_record(&connection, node, &debug)?;
    let (image, exported) = review(&connection, node, &debug)?;
    Ok((
        connection,
        ClientReport {
            turns,
            image,
            exported,
        },
    ))
}

/// Find the counter button and check that `get_by_role` returns a node of
/// `get_tree` that `get_node` reads. Returns the node and the error context.
fn inspect_button(connection: &Connection) -> Result<(p::NodeId, String), String> {
    let found = call::<p::GetByRole>(
        connection,
        p::RoleQuery {
            window: 1,
            within: None,
            role: p::Role::Button,
            name: Some("Increment".to_owned()),
            timeout_ms: None,
        },
        "get_by_role",
        "",
    )?;
    let node = found.node.id;
    let tree = call::<p::GetTree>(
        connection,
        p::GetTreeParams {
            window: 1,
            root: None,
            depth: None,
        },
        "get_tree",
        "",
    )?;
    let in_tree = tree.nodes.iter().any(|info| info.id == node);
    let detail = call::<p::GetNode>(connection, p::GetNodeParams { node }, "get_node", "")?;
    let debug = format!(
        "node {node} after get_by_role: in get_tree = {in_tree}; get_node = ok (role {:?}, name {:?})",
        detail.node.role, detail.node.name
    );
    if !in_tree {
        return Err(format!("get_tree does not contain {debug}"));
    }
    Ok((node, debug))
}

/// Highlight and override the button, then record and stop one click.
fn override_and_record(
    connection: &Connection,
    node: p::NodeId,
    debug: &str,
) -> Result<p::TurnList, String> {
    call::<p::Highlight>(
        connection,
        p::HighlightParams {
            node: Some(node),
            layout: false,
        },
        "highlight",
        debug,
    )?;
    call::<p::SetStyleOverride>(
        connection,
        p::SetStyleOverrideParams {
            scope: p::OverrideScope::Node { node },
            property: "padding".to_owned(),
            value: "20".to_owned(),
        },
        "set_style_override",
        debug,
    )?;
    let latest = call::<p::ListTurns>(
        connection,
        p::ListTurnsParams {
            since: None,
            limit: Some(1),
            wait_ms: None,
        },
        "list_turns (before recording)",
        debug,
    )?;
    let since = latest.turns.last().map(|turn| turn.turn);
    call::<p::StartRecording>(
        connection,
        p::RecordingSettings {
            updates: true,
            frames: false,
            input: false,
        },
        "start_recording",
        debug,
    )?;
    call::<p::InjectInput>(
        connection,
        p::InjectInputParams {
            window: 1,
            steps: vec![p::InputStep::Click { node }],
        },
        "inject_input",
        debug,
    )?;
    call::<p::StopRecording>(connection, p::Empty {}, "stop_recording", debug)?;
    call::<p::ListTurns>(
        connection,
        p::ListTurnsParams {
            since,
            limit: None,
            wait_ms: None,
        },
        "list_turns (after recording)",
        debug,
    )
}

/// Comment on the button with a screenshot and export the review.
fn review(
    connection: &Connection,
    node: p::NodeId,
    debug: &str,
) -> Result<(Option<String>, p::ExportedReview), String> {
    let item = call::<p::AddComment>(
        connection,
        p::AddCommentParams {
            window: 1,
            node: Some(node),
            text: "the counter button".to_owned(),
            screenshot: true,
        },
        "add_comment",
        debug,
    )?;
    let exported = call::<p::ExportReview>(connection, p::Empty {}, "export_review", debug)?;
    Ok((item.image, exported))
}

/// One typed call, with the method name and the state of the inspected node
/// in the error.
fn call<M: p::Method>(
    connection: &Connection,
    params: M::Params,
    context: &str,
    debug: &str,
) -> Result<M::Result, String> {
    connection
        .call::<M>(params)
        .map_err(|error| format!("{context}: {error} ({debug})"))
}
