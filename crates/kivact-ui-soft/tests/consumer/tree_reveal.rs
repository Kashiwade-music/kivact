//! Role: Verify that tree branch exits retain paint while logical rows disappear.

use kivact::interaction::TreeNode;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::style::rgb;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::view::Key;
use kivact::view::Role;
use kivact::view::node;
use std::time::Duration;

#[test]
fn branch_exit_keeps_its_paint_but_removes_the_child_from_navigation() {
    let screen = render_with(
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            kivact::ui::tree(
                "Files",
                vec![
                    TreeNode::new(1, None, "Folder"),
                    TreeNode::new(2, Some(1), "Document"),
                    TreeNode::new(3, None, "Sibling"),
                ],
            )
            .expanded(signal(vec![1]))
            .reveal(
                2.0,
                kivact::motion::Transition::tween(
                    734.77,
                    kivact::motion::Easing::CubicBezier(0.327, 0.694, 0.116, 1.0),
                ),
            )
            .item(|row, _| {
                node().style(move || {
                    style().size(40, 40).background(if row.get().key == 2 {
                        rgb(0xcc0000)
                    } else {
                        rgb(0xdddddd)
                    })
                })
            })
        },
    );
    let folder = || screen.get_by_role(Role::TreeItem, "Folder");
    let sibling_y = || {
        screen
            .get_by_role(Role::TreeItem, "Sibling")
            .rect()
            .unwrap()
            .y
    };
    let previous_y = sibling_y();
    assert!(screen.screenshot().pixels.contains(&0xcc0000));
    folder().press_key(Key::ArrowLeft);
    assert!(screen.query_by_role(Role::TreeItem, "Document").is_none());
    assert!(screen.screenshot().pixels.contains(&0xcc0000));
    assert_eq!(sibling_y(), previous_y);
    folder().press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::TreeItem, "Sibling").is_focused());

    screen.advance(Duration::from_millis(120));
    let intermediate_y = sibling_y();
    assert!(intermediate_y < previous_y);
    screen.advance(Duration::from_millis(650));
    assert!(sibling_y() < intermediate_y);
    assert!(!screen.screenshot().pixels.contains(&0xcc0000));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
