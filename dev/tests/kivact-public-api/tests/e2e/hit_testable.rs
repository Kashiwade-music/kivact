//! Role: Tests that a node can let the pointer pass through it.
//!
//! `.hit_testable(false)` on real windows: a card drawn over a button (a
//! tooltip that follows the pointer) takes no press or hover, so both reach
//! the button behind it, and its children are passed through too.

use crate::support::mount;
use kivact::prelude::*;
use kivact::style::Position;

#[test]
fn a_node_that_is_not_hit_testable_lets_the_pointer_reach_what_lies_behind() {
    let (screen, (presses, hovered)) = mount(|| {
        let presses = signal(0);
        let target = node_ref();
        let hovered = target.hovered();
        let view = node().style(style().size(300, 200)).child((
            node()
                .role(Role::Button)
                .name("Behind")
                .node_ref(target)
                .on_press(move || presses.update(|n| *n += 1))
                .style(style().size(200, 100)),
            node()
                .hit_testable(false)
                .test_id("card")
                .style(
                    style()
                        .position(Position::Absolute)
                        .left(20)
                        .top(20)
                        .size(100, 50),
                )
                .child(node().test_id("inner").style(style().size(80, 30))),
        ));
        ((presses, hovered), view)
    });
    let r = screen.get_by_test_id("inner").rect().expect("laid out");
    screen.click_at(r.x + 10.0, r.y + 10.0);
    assert_eq!(
        presses.get_untracked(),
        1,
        "the press passed through the card"
    );
    assert!(hovered.get_untracked(), "and so did the hover");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
