//! Role: Tests that the draw cache and the paint list keep their invariants:
//! a stale kept drawing is caught, and scrolling keeps the list bounded.

use kivact_engine::api::Client;
use kivact_engine::contract::{
    BoxData, DrawableMode, DrawableRole, ExportRequestMode, HostRequest, NodeKind, TurnOutput,
};
use kivact_engine::testing::{ArvaneTestHost, MirrorMode, TestHostOptions};
use kivact_types::geometry3d::Matrix4;
use kivact_types::paint::Fill;
use kivact_types::{
    DrawableId, ElementTransform, LayoutProps, Length, NodeId, Overflow, PaintProps, Point, Rgba,
    WillChange, WindowId,
};

const ROOT: NodeId = NodeId(1);
const LIST: NodeId = NodeId(2);
/// How many rows the list scrolls: 20 px each in a list 200 px tall.
const ROWS: u64 = 200;

/// Row `i` of the list.
fn row(i: u64) -> NodeId {
    NodeId(100 + i)
}

fn painted(hex: u32) -> PaintProps {
    PaintProps {
        background: Some(Fill::Solid(Rgba::from_hex(hex, 1.0))),
        ..PaintProps::default()
    }
}

/// A row's paint: even rows are compositor layers of their own, odd rows
/// are drawn through a transform (the list keeps both kinds).
fn row_paint(i: u64, hex: u32) -> PaintProps {
    if i.is_multiple_of(2) {
        PaintProps {
            will_change: WillChange::Transform,
            ..painted(hex)
        }
    } else {
        PaintProps {
            transform: ElementTransform {
                scale: [0.9, 1.0, 1.0],
                ..ElementTransform::default()
            },
            ..painted(hex)
        }
    }
}

fn sized(width: f64, height: f64) -> LayoutProps {
    LayoutProps {
        width: Length::Px(width),
        height: Length::Px(height),
        ..LayoutProps::default()
    }
}

/// A 200 × 200 list scrolling `ROWS` rows.
fn start() -> ArvaneTestHost {
    ArvaneTestHost::new(
        Client::new(
            |_, _, writer| {
                writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                writer.create(WindowId::MAIN, LIST, NodeKind::Box(BoxData::default()));
                writer.insert(ROOT, LIST, None);
                writer.set_layout(
                    LIST,
                    LayoutProps {
                        overflow_y: Overflow::Scroll,
                        ..sized(200.0, 200.0)
                    },
                );
                for i in 0..ROWS {
                    writer.create(WindowId::MAIN, row(i), NodeKind::Box(BoxData::default()));
                    writer.insert(LIST, row(i), None);
                    writer.set_layout(
                        row(i),
                        LayoutProps {
                            shrink: 0.0,
                            ..sized(180.0, 20.0)
                        },
                    );
                    writer.set_paint(row(i), row_paint(i, 0x2563eb));
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
    .expect("the test host starts")
}

/// Scroll the list to `y` and draw the frame.
fn scroll_to(host: &mut ArvaneTestHost, y: f64) {
    host.writer().scroll_to(LIST, Point { x: 0.0, y });
    host.writer().commit();
    host.wake();
    host.redraw();
}

/// A paint change whose redraw mark is left out (an injected fault) is
/// caught by the check of the kept drawing, which draws from the node's
/// paint now, not the kept one (the check runs in debug builds with
/// `testing` only).
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "is stale")]
fn a_paint_change_that_does_not_forget_the_drawing_is_caught() {
    let mut host = start();
    host.redraw();
    host.keep_stale_draw(Some(row(0)));
    host.writer().set_paint(row(0), row_paint(0, 0xdc2626));
    host.writer().commit();
    host.wake();
    host.redraw();
}

/// Scrolling down and back 10 000 times walks the list's subtree again
/// each time; at the top the list is the size it was in the first frame.
#[test]
fn scrolling_back_and_forth_keeps_the_paint_list_the_size_it_was() {
    let mut host = start();
    host.redraw();
    let top = host.paint_list_sizes(WindowId::MAIN);
    assert!(
        top.transforms > 0 && top.layers > 0,
        "the rows hold transforms and layers: {top:?}"
    );
    for round in 1..=10_000 {
        scroll_to(&mut host, 400.0);
        scroll_to(&mut host, 0.0);
        if round % 1_000 == 0 {
            assert_eq!(
                host.paint_list_sizes(WindowId::MAIN),
                top,
                "after {round} round trips"
            );
        }
    }
}

/// Rows scrolled out of the list drop their kept drawings: scrolling
/// through every row keeps no more drawings than the list has entries.
#[test]
fn rows_scrolled_out_drop_their_kept_drawings() {
    let mut host = start();
    host.redraw();
    for step in 1..=38_u32 {
        scroll_to(&mut host, f64::from(step) * 100.0);
        let sizes = host.paint_list_sizes(WindowId::MAIN);
        assert!(
            sizes.kept_draws <= sizes.entries,
            "at {} px: {sizes:?}",
            step * 100
        );
    }
}

const DRAWABLE: NodeId = NodeId(3);
const DRAWABLE_ID: DrawableId = DrawableId(1);

/// A host with one export drawable (100 × 100) under the root.
fn start_with_drawable() -> ArvaneTestHost {
    ArvaneTestHost::new(
        Client::new(
            |_, _, writer| {
                writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                writer.create(WindowId::MAIN, DRAWABLE, NodeKind::Box(BoxData::default()));
                writer.insert(ROOT, DRAWABLE, None);
                writer.set_layout(DRAWABLE, sized(100.0, 100.0));
                writer.set_drawable(
                    DRAWABLE,
                    Some(DrawableRole {
                        id: DRAWABLE_ID,
                        mode: DrawableMode::Export,
                    }),
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

/// The identity moved by `x` logical px along the x axis.
fn moved_by(x: f64) -> Matrix4 {
    let mut m = [[0.0; 4]; 4];
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    m[0][3] = x;
    Matrix4 { m }
}

/// The geometry the app writes for a drawable is applied and reported by
/// the write's own pass: a wake after the frame syncs nothing, presents no
/// frame and leaves the paint list the size it was.
#[test]
fn a_drawable_geometry_write_leaves_no_report_pending() {
    let mut host = start_with_drawable();
    host.writer().set_engine_recording(true, true);
    host.redraw();
    host.writer().drawable_request(HostRequest::DrawableExport {
        id: DRAWABLE_ID,
        generation: 1,
        mode: ExportRequestMode::None,
        scale: None,
    });
    host.writer()
        .drawable_request(HostRequest::SetDrawableGeometry {
            id: DRAWABLE_ID,
            generation: 1,
            geometry: Some(moved_by(10.0)),
        });
    host.writer().commit();
    host.wake();
    host.redraw();
    let settled = host.paint_list_sizes(WindowId::MAIN);
    let _ = host.writer().reader().take_engine_records();
    host.wake();
    let after = host.writer().reader().take_engine_records();
    assert!(
        after.syncs.is_empty(),
        "a wake after the write synced again: {:?}",
        after.syncs
    );
    assert!(
        after.frames.is_empty(),
        "a wake after the write presented a frame: {:?}",
        after.frames
    );
    assert_eq!(host.paint_list_sizes(WindowId::MAIN), settled);
}
