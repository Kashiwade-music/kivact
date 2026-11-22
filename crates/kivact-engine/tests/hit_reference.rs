//! Role: Tests that the self-check's reference walk does not share hit candidates.

use kivact_engine::api::Client;
use kivact_engine::contract::{BoxData, NodeKind, TurnOutput};
use kivact_engine::testing::{ArvaneTestHost, MirrorMode, TestHostOptions};
use kivact_types::{LayoutProps, Length, NodeId, WindowId};

const ROOT: NodeId = NodeId(1);
const CHILD: NodeId = NodeId(2);

/// A root and one child box covering the whole window.
fn start() -> ArvaneTestHost {
    ArvaneTestHost::new(
        Client::new(
            |_, _, writer| {
                writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                writer.create(WindowId::MAIN, CHILD, NodeKind::Box(BoxData::default()));
                writer.insert(ROOT, CHILD, None);
                writer.set_layout(
                    CHILD,
                    LayoutProps {
                        width: Length::Percent(100.0),
                        height: Length::Percent(100.0),
                        ..LayoutProps::default()
                    },
                );
                writer.commit();
            },
            |_, _, _| TurnOutput::default(),
        ),
        TestHostOptions {
            mirror: MirrorMode::Off,
            ..TestHostOptions::default()
        },
    )
    .expect("the test host starts")
}

/// The self-check's hit errors.
fn hit_errors(host: &mut ArvaneTestHost) -> Vec<String> {
    host.self_check()
        .into_iter()
        .filter(|error| error.starts_with("hit at"))
        .collect()
}

/// The hit test's candidates leave out the child under the window's
/// center (an injected fault): the complete reference walk still finds
/// it, and the self-check reports the difference.
#[cfg(debug_assertions)]
#[test]
fn the_reference_hit_walk_finds_a_child_the_candidates_left_out() {
    let mut host = start();
    host.redraw();
    assert!(
        hit_errors(&mut host).is_empty(),
        "without a fault the walks agree"
    );
    host.drop_hit_candidate(Some(CHILD));
    host.writer().commit();
    host.wake();
    host.redraw();
    let found = hit_errors(&mut host);
    host.drop_hit_candidate(None);
    assert!(
        !found.is_empty(),
        "the reference walk must not share the candidates"
    );
}
