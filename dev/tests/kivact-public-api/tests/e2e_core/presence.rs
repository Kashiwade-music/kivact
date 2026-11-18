//! Role: Verify reactive node presence through entrance and removal.

use std::time::Duration;

use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::view::Presence;

use kivact::interaction::{TreeNode, use_tree};
use kivact::test::Screen;
use kivact_public_api_tests::support::mount;

#[test]
fn presence_reports_entering_present_leaving() {
    let (screen, (shown, presence)) = mount(|| {
        let shown = signal(true);
        let target = node_ref();
        let presence = target.presence();
        let panel = show(shown, move || {
            node().node_ref(target).style(
                style()
                    .size(40, 30)
                    .enter(style().opacity(0.0))
                    .exit(style().opacity(0.0))
                    .transition(TransitionProp::All, Transition::tween(200, Easing::Linear)),
            )
        });
        ((shown, presence), panel)
    });
    assert_eq!(presence.get(), Presence::Entering);
    screen.advance(Duration::from_millis(250));
    assert_eq!(presence.get(), Presence::Present);
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(presence.get(), Presence::Leaving);
    screen.advance(Duration::from_millis(250));
    assert_eq!(presence.get(), Presence::Leaving);
    shown.set(true);
    screen.run_until_idle();
    assert_eq!(presence.get(), Presence::Entering);
    screen.advance(Duration::from_millis(250));
    assert_eq!(presence.get(), Presence::Present);
}

#[test]
fn exit_size_collapses_the_node_before_it_is_removed() {
    use kivact::view::Orientation;

    let (screen, (shown, presence)) = mount(|| {
        let shown = signal(true);
        let target = node_ref();
        let presence = target.presence();
        let panel = column((
            show(shown, move || {
                node().node_ref(target).test_id("collapsing").style(
                    style()
                        .size(80, 100)
                        .enter_size(Orientation::Vertical)
                        .exit_size(Orientation::Vertical)
                        .transition(TransitionProp::Size, Transition::tween(200, Easing::Linear)),
                )
            }),
            node().test_id("after").style(style().size(20, 20)),
        ))
        .test_id("size-parent");
        ((shown, presence), panel)
    });
    assert_eq!(presence.get(), Presence::Entering);
    screen.advance(Duration::from_millis(250));
    assert_eq!(presence.get(), Presence::Present);
    let panel = screen.get_by_test_id("collapsing");
    let original = panel.rect().expect("laid out").height;
    let parent_height = screen
        .get_by_test_id("size-parent")
        .rect()
        .expect("laid out")
        .height;
    let before = screen.get_by_test_id("after").rect().expect("laid out").y;
    shown.set(false);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    let height = panel.rect().expect("retained during exit").height;
    assert!(
        height > 0.0 && height < original,
        "intermediate height: {height}"
    );
    let during = screen.get_by_test_id("after").rect().expect("laid out").y;
    assert!(during < before && during > before - original);
    assert!(
        screen
            .get_by_test_id("size-parent")
            .rect()
            .expect("laid out")
            .height
            < parent_height
    );
    screen.advance(Duration::from_millis(150));
    assert!(panel.rect().is_none(), "removed when size motion finishes");
}

fn advance(screen: &Screen, ms: u64) {
    screen.advance(Duration::from_millis(ms));
}

#[test]
fn disclosure_unmounts_closed_content_after_its_exit() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            column((
                kivact::ui::disclosure("Details", open)
                    .animated(true)
                    .content(|| {
                        node()
                            .test_id("content")
                            .style(style().size(100, 80).background(rgb(0xcc0000)))
                    }),
                node().test_id("after").style(style().size(100, 20)),
            )),
        )
    });
    advance(&screen, 300);
    let expanded = screen.get_by_test_id("after").rect().unwrap().y;
    open.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_test_id("content").is_none());
    assert_eq!(
        screen.get_by_role(Role::Button, "Details").is_expanded(),
        Some(false)
    );
    let start = screen.get_by_test_id("after").rect().unwrap().y;
    advance(&screen, 80);
    let middle = screen.get_by_test_id("after").rect().unwrap().y;
    advance(&screen, 300);
    let end = screen.get_by_test_id("after").rect().unwrap().y;
    assert!(start >= expanded - 1.0 && start > middle && middle > end);
    assert!(!screen.screenshot().pixels.contains(&0xcc0000));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tree_row_reveal_expands_with_enter_size() {
    let (screen, expanded) = mount(|| {
        let expanded = signal(Vec::<i32>::new());
        let tree = use_tree(
            vec![
                TreeNode::new(1, None, "Folder"),
                TreeNode::new(2, Some(1), "Child"),
                TreeNode::new(3, None, "Sibling"),
            ],
            expanded,
            signal(None),
        );
        let rows = kivact::ui::tree_rows(
            tree,
            |_| (),
            move |key, _, part| {
                node()
                    .part(part)
                    .name(tree.label(key))
                    .style(style().size(100, 40))
            },
        )
        .reveal(0.0, Transition::tween(200, Easing::Linear));
        (expanded, column(rows).part(tree.list()))
    });
    advance(&screen, 300);
    let y = || {
        screen
            .get_by_role(Role::TreeItem, "Sibling")
            .rect()
            .unwrap()
            .y
    };
    let closed = y();
    expanded.set(vec![1]);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::TreeItem, "Child").is_some());
    advance(&screen, 80);
    let middle = y();
    advance(&screen, 300);
    assert!(closed < middle && middle < y());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
