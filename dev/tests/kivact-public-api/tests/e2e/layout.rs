//! Role: Verify transparent boxes in real window layout and semantics.

use kivact::prelude::*;
use kivact::style::Display;
use kivact::test::render_with;

use crate::layout_units::{assert_rect, rect_of, window};

#[test]
fn display_contents_lays_out_children_in_the_parent() {
    let screen = render_with(window(400.0, 200.0), || {
        row((
            node().test_id("before").style(style().width(30).height(20)),
            node()
                .test_id("contents")
                .role(Role::Group)
                .name("transparent")
                .style(style().display(Display::Contents).padding(50).width(300))
                .child((
                    node()
                        .test_id("first")
                        .role(Role::Button)
                        .name("first")
                        .style(style().width(40).height(20)),
                    node()
                        .test_id("second")
                        .role(Role::Button)
                        .name("second")
                        .style(style().width(50).height(20)),
                )),
            node().test_id("after").style(style().width(60).height(20)),
        ))
        .test_id("parent")
        .role(Role::Group)
        .name("parent")
        .style(style().gap(10))
    });
    assert_rect(
        rect_of(&screen, "first"),
        (40.0, 0.0, 40.0, 20.0),
        "first flattened item",
    );
    assert_rect(
        rect_of(&screen, "second"),
        (90.0, 0.0, 50.0, 20.0),
        "second flattened item",
    );
    assert_rect(
        rect_of(&screen, "after"),
        (150.0, 0.0, 60.0, 20.0),
        "following sibling",
    );
    let tree = screen.accessibility_tree();
    assert!(!tree.contains("#contents"), "{tree}");
    let indent = |test_id: &str| {
        let line = tree
            .lines()
            .find(|line| line.contains(test_id))
            .expect("invariant: named semantic node is published");
        line.len() - line.trim_start().len()
    };
    assert_eq!(indent("#first"), indent("#parent") + 2, "{tree}");
    assert_eq!(indent("#second"), indent("#parent") + 2, "{tree}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
