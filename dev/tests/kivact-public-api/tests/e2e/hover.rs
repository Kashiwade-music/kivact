//! Role: Tests for hover state tracking within nodes.
//!
//! `NodeRef::hovered`: whether the host's hit is the node or in its
//! subtree, whatever pointer listener or capture there is. It is false
//! outside the window (also with a button held), after the platform
//! cancelled the pointer, and once the hit is hidden, removed or outside an
//! open modal.


use crate::support::mount;
use kivact::geometry::Point;
use kivact::prelude::*;
use kivact::view::{Handled, Placement};
use kivact::view::{MouseButton, overlay_at};

struct Handles {
    over: Value<bool>,
    hidden: Signal<bool>,
    mounted: Signal<bool>,
    modal: Signal<bool>,
}

#[test]
fn hovered_follows_the_hit_through_listeners_capture_and_tree_changes() {
    let (screen, h) = mount(|| {
        let (hidden, mounted, modal) = (signal(false), signal(true), signal(false));
        let group = node_ref();
        let over = group.hovered();
        let view = column((
            column((
                node()
                    .test_id("plain")
                    .style(style().size(100, 30).shrink(0.0)),
                show(mounted, || {
                    node()
                        .test_id("listener")
                        .on_pointer(|_| Handled::Yes)
                        .style(style().size(100, 30).shrink(0.0))
                }),
            ))
            .node_ref(group)
            .hidden(hidden)
            .test_id("group"),
            node()
                .test_id("outside")
                .style(style().size(100, 30).shrink(0.0)),
            show(modal, || {
                overlay_at(Point::new(400.0, 400.0), Placement::Center, text("notice"))
                    .modal()
                    .role(Role::Dialog)
            }),
        ));
        (
            Handles {
                over,
                hidden,
                mounted,
                modal,
            },
            view,
        )
    });
    let over = || h.over.get_untracked();
    assert!(!over(), "the pointer has not entered yet");
    screen.get_by_test_id("plain").hover();
    assert!(over(), "over a plain child");
    screen.get_by_test_id("listener").hover();
    assert!(
        over(),
        "a child with its own on_pointer does not hide the hit"
    );
    screen.get_by_test_id("outside").hover();
    assert!(!over(), "over a node outside the subtree");

    screen.get_by_test_id("listener").hover();
    screen.mouse_leave();
    assert!(!over(), "outside the window");
    screen.get_by_test_id("listener").hover();
    screen.set_window_focused(false);
    assert!(!over(), "the platform cancelled the pointer");
    screen.set_window_focused(true);

    let listener = screen.get_by_test_id("listener").rect().expect("laid out");
    screen.mouse_move(listener.center().x, listener.center().y);
    screen.mouse_down(MouseButton::Primary);
    assert!(over(), "pressed, the listener captures the pointer");
    screen.mouse_leave();
    assert!(
        !over(),
        "the capture outlives the window, the hover does not"
    );
    screen.mouse_up(MouseButton::Primary);

    screen.mouse_move(listener.center().x, listener.center().y);
    assert!(over());

    h.hidden.set(true);
    screen.run_until_idle();
    assert!(!over(), "the hit got hidden");
    h.hidden.set(false);
    screen.run_until_idle();
    screen.mouse_move(listener.center().x, listener.center().y + 1.0);
    assert!(over());

    h.mounted.set(false);
    screen.run_until_idle();
    assert!(!over(), "the hit was removed: its ancestors are no new hit");
    h.mounted.set(true);
    screen.run_until_idle();
    screen.mouse_move(listener.center().x, listener.center().y);
    assert!(over());

    h.modal.set(true);
    screen.run_until_idle();
    assert!(!over(), "the hit lies outside the open modal");
    screen.mouse_move(listener.center().x + 1.0, listener.center().y);
    assert!(!over(), "behind the modal's backdrop nothing is hit");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A row of two cells: the row's hover style colors its background, the
/// first cell is hidden until the row (its group) is hovered.
#[test]
fn hover_covers_the_children_and_group_hover_styles_a_child_by_its_groups_hover() {
    use crate::paint::{distance, pixel};
    use kivact::style::rgb;
    const ROW: u32 = 0x0000ff;
    const ACTIONS: u32 = 0xff0000;
    let (screen, ()) = mount(|| {
        let cell = |id: &'static str| node().style(style().size(100, 40)).test_id(id);
        let view = column((
            row((
                cell("actions").style(
                    style()
                        .background(rgb(ACTIONS))
                        .opacity(0.0)
                        .group_hover(style().opacity(1.0)),
                ),
                cell("name"),
                // A nested group: what is below it follows its own hover.
                row(cell("inner").style(
                    style()
                        .background(rgb(ACTIONS))
                        .opacity(0.0)
                        .group_hover(style().opacity(1.0)),
                ))
                .style(style().group())
                .test_id("nested"),
            ))
            .style(
                style()
                    .group()
                    .background(rgb(0xffffff))
                    .hover(style().background(rgb(ROW))),
            )
            .test_id("row"),
            node().style(style().size(100, 40)).test_id("outside"),
        ));
        ((), view)
    });
    let at = |id: &str| {
        let r = screen.get_by_test_id(id).rect().expect("laid out");
        pixel(
            &screen.screenshot(),
            (r.x + 50.0) as u32,
            (r.y + 20.0) as u32,
        )
    };
    let page = at("outside");
    assert_eq!(at("actions"), at("name"), "hidden until the row is hovered");
    // The pointer over a child: the row is hovered (CSS `:hover`), and its
    // group hover shows the first cell; the nested group is not hovered.
    screen.get_by_test_id("name").hover();
    assert!(
        distance(at("name"), ROW) <= 8,
        "the row's hover covers its children"
    );
    assert!(
        distance(at("actions"), ACTIONS) <= 8,
        "shown by the row's hover"
    );
    assert!(
        distance(at("inner"), ROW) <= 8,
        "the nested group is not hovered"
    );
    screen.get_by_test_id("inner").hover();
    assert!(
        distance(at("inner"), ACTIONS) <= 8,
        "shown by its own group's hover"
    );
    assert!(
        distance(at("actions"), ACTIONS) <= 8,
        "the row is still hovered"
    );
    screen.get_by_test_id("outside").hover();
    assert_eq!(
        at("actions"),
        page,
        "hidden again once the pointer leaves the row"
    );
    assert!(distance(at("name"), 0xffffff) <= 8);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
