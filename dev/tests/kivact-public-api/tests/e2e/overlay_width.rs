//! Role: Tests that an overlay can be at least as wide as its anchor.
//!
//! `Overlay::at_least_anchor_width` on real windows: a narrow overlay
//! takes its anchor's width in the frame it opens, a wider one keeps its
//! own, it follows the anchor when the anchor's width changes, and an
//! overlay without the option or placed at a point keeps its content's
//! width.

use crate::support::mount;
use kivact::draw::Point;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::view::{Placement, overlay, overlay_at};

fn content(width: f32) -> Node {
    node().style(style().size(width, 20))
}

#[test]
fn an_overlay_is_at_least_as_wide_as_its_anchor_when_asked() {
    let (screen, (anchor_width, open)) = mount(|| {
        let anchor_width = signal(200.0f32);
        let open = signal(false);
        let anchor = node_ref();
        let view = column((
            node()
                .node_ref(anchor)
                .style(move || style().size(anchor_width.get(), 30)),
            show(open, move || {
                (
                    overlay(anchor, Placement::Below, content(50.0))
                        .at_least_anchor_width(true)
                        .test_id("narrow"),
                    overlay(anchor, Placement::Below, content(300.0))
                        .at_least_anchor_width(true)
                        .test_id("wide"),
                    overlay(anchor, Placement::Below, content(50.0)).test_id("plain"),
                    overlay_at(Point::new(10.0, 10.0), Placement::Below, content(50.0))
                        .at_least_anchor_width(true)
                        .test_id("point"),
                )
            }),
        ))
        .style(style().align_items(Align::Start));
        ((anchor_width, open), view)
    });
    open.set(true);
    // One turn: the overlay has the anchor's width in the frame it opens.
    screen.run_until_idle();
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    assert_eq!(width("narrow"), 200.0);
    assert_eq!(width("wide"), 300.0);
    assert_eq!(width("plain"), 50.0);
    assert_eq!(width("point"), 50.0);
    anchor_width.set(120.0);
    screen.run_until_idle();
    assert_eq!(width("narrow"), 120.0, "it follows the anchor");
    assert_eq!(width("wide"), 300.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
