//! Role: Scroll-driven progress: a scroller's offset and a node's view progress.

use crate::support::mount;
use kivact::geometry::Point;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::{ScrollBehavior, ViewRange, scroll_progress, view_progress};

/// Scroll container of 100 px high over 400 px of content (a 300 px range),
/// with a 40 px target starting at y = 200 in the content. The container is
/// at the top of the window, so the target's window y is `200 - offset`.
/// The progress values are made in setup, as the API requires: the scroll
/// progress, then the four view ranges (cover, contain, entry, exit).
fn fixture() -> (Screen, NodeRef, Vec<Value<f64>>) {
    let (screen, (scroller, values)) = mount(|| {
        let scroller = node_ref();
        let target = node_ref();
        let values = vec![
            scroll_progress(scroller, Orientation::Vertical),
            view_progress(target, scroller, Orientation::Vertical, ViewRange::Cover),
            view_progress(target, scroller, Orientation::Vertical, ViewRange::Contain),
            view_progress(target, scroller, Orientation::Vertical, ViewRange::Entry),
            view_progress(target, scroller, Orientation::Vertical, ViewRange::Exit),
        ];
        (
            (scroller, values),
            scroll(column((
                node().style(style().size(200, 200)),
                node().node_ref(target).style(style().size(200, 40)),
                node().style(style().size(200, 160)),
            )))
            .node_ref(scroller)
            .style(style().size(200, 100)),
        )
    });
    (screen, scroller, values)
}

fn scroll_to(scroller: NodeRef, screen: &Screen, y: f64) {
    scroller.scroll_to(Point::new(0.0, y), ScrollBehavior::Instant);
    screen.run_until_idle();
}

#[test]
fn scroll_progress_tracks_offset() {
    let (screen, scroller, values) = fixture();
    let progress = values[0];
    assert_eq!(progress.get_untracked(), 0.0, "at the start");
    scroll_to(scroller, &screen, 75.0);
    assert!(
        (progress.get_untracked() - 0.25).abs() < 1e-9,
        "75 of a 300 px range: {}",
        progress.get_untracked()
    );
    scroll_to(scroller, &screen, 300.0);
    assert_eq!(progress.get_untracked(), 1.0, "at the end of the range");
    scroll_to(scroller, &screen, 100_000.0);
    assert_eq!(progress.get_untracked(), 1.0, "past the end stays at 1");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The four view ranges of §12.8, for a target of T = 40 px in a viewport
/// V = 100 px (T ≤ V): `r` is the target's start relative to the viewport's
/// start, `r = 200 - offset`. Expected values are the table's formulas
/// evaluated at each `r`, clamped to 0..=1.
#[test]
fn view_progress_ranges() {
    let (screen, scroller, values) = fixture();
    let (cover, contain, entry, exit) = (values[1], values[2], values[3], values[4]);
    // (offset, r, cover, contain, entry, exit)
    let rows = [
        (100.0, 100.0, 0.0, 0.0, 0.0, 0.0),
        (120.0, 80.0, 20.0 / 140.0, 0.0, 0.5, 0.0),
        (170.0, 30.0, 0.5, 0.5, 1.0, 0.0),
        (200.0, 0.0, 100.0 / 140.0, 1.0, 1.0, 0.0),
        (220.0, -20.0, 120.0 / 140.0, 1.0, 1.0, 0.5),
        (240.0, -40.0, 1.0, 1.0, 1.0, 1.0),
    ];
    for (offset, r, c, ct, en, ex) in rows {
        scroll_to(scroller, &screen, offset);
        for (name, actual, expected) in [
            ("cover", cover.get_untracked(), c),
            ("contain", contain.get_untracked(), ct),
            ("entry", entry.get_untracked(), en),
            ("exit", exit.get_untracked(), ex),
        ] {
            assert!(
                (actual - expected).abs() < 1e-6,
                "{name} at r = {r}: expected {expected}, got {actual}"
            );
        }
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
