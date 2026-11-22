//! Role: Tests that an unchanged layer keeps its content version; a changed one bumps it.

use arvane_protocol::LayerKey;
use kivact_engine::api::Client;
use kivact_engine::contract::{BoxData, NodeKind, TurnOutput};
use kivact_engine::testing::{ArvaneTestHost, MirrorMode, TestHostOptions};
use kivact_types::paint::Fill;
use kivact_types::{LayoutProps, Length, NodeId, PaintProps, Rgba, WillChange, WindowId};

const ROOT: NodeId = NodeId(1);

fn painted(hex: u32) -> PaintProps {
    PaintProps {
        background: Some(Fill::Solid(Rgba::from_hex(hex, 1.0))),
        ..PaintProps::default()
    }
}

fn root_version(host: &ArvaneTestHost) -> u64 {
    host.layer_versions(WindowId::MAIN)
        .into_iter()
        .find(|(key, _)| *key == LayerKey::Root)
        .map(|(_, version)| version)
        .expect("the root layer is drawn")
}

#[test]
fn an_unchanged_layer_keeps_its_content_version_and_a_changed_one_bumps_it() {
    let mut host = ArvaneTestHost::new(
        Client::new(
            |_, _, writer| {
                writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                writer.set_paint(ROOT, painted(0x2563eb));
                writer.commit();
            },
            |_, _, _| TurnOutput::default(),
        ),
        TestHostOptions {
            mirror: MirrorMode::Off,
            ..TestHostOptions::default()
        },
    )
    .expect("the test host starts");
    host.redraw();
    let first = root_version(&host);

    // The same paint is no write: the next frame keeps the version.
    host.writer().set_paint(ROOT, painted(0x2563eb));
    host.writer().commit();
    host.wake();
    host.redraw();
    assert_eq!(root_version(&host), first, "nothing in the layer changed");

    // A new background is drawn again: the version moves on.
    host.writer().set_paint(ROOT, painted(0xdc2626));
    host.writer().commit();
    host.wake();
    assert!(host.redraw(), "the change asks for a frame");
    assert!(root_version(&host) > first, "the root's background changed");
}

const A: NodeId = NodeId(2);
const B: NodeId = NodeId(3);

/// The version of the layer `key` in the last frame.
fn version_of(host: &ArvaneTestHost, key: LayerKey) -> u64 {
    host.layer_versions(WindowId::MAIN)
        .into_iter()
        .find(|(k, _)| *k == key)
        .map(|(_, version)| version)
        .expect("the layer is drawn")
}

/// A 100 × 100 box painted `hex` that is a compositor layer of its own.
fn layer_painted(hex: u32) -> PaintProps {
    PaintProps {
        will_change: WillChange::Transform,
        ..painted(hex)
    }
}

#[test]
fn a_changed_layer_bumps_its_version_and_a_sibling_layer_keeps_its_own() {
    let mut host = ArvaneTestHost::new(
        Client::new(
            |_, _, writer| {
                writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                for (id, hex) in [(A, 0x2563eb), (B, 0x16a34a)] {
                    writer.create(WindowId::MAIN, id, NodeKind::Box(BoxData::default()));
                    writer.insert(ROOT, id, None);
                    writer.set_layout(
                        id,
                        LayoutProps {
                            width: Length::Px(100.0),
                            height: Length::Px(100.0),
                            ..LayoutProps::default()
                        },
                    );
                    writer.set_paint(id, layer_painted(hex));
                }
                writer.commit();
            },
            |_, _, _| TurnOutput::default(),
        ),
        TestHostOptions {
            mirror: MirrorMode::Off,
            ..TestHostOptions::default()
        },
    )
    .expect("the test host starts");
    host.redraw();
    let (a, b) = (
        version_of(&host, LayerKey::Node(A)),
        version_of(&host, LayerKey::Node(B)),
    );

    // A's new background draws A again; B is replayed (and, in debug
    // builds with `testing`, checked) without a new command.
    host.writer().set_paint(A, layer_painted(0xdc2626));
    host.writer().commit();
    host.wake();
    assert!(host.redraw(), "the change asks for a frame");
    assert!(
        version_of(&host, LayerKey::Node(A)) > a,
        "A's background changed"
    );
    assert_eq!(
        version_of(&host, LayerKey::Node(B)),
        b,
        "nothing in B changed"
    );
}
