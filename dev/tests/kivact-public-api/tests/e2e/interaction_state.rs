//! Role: Behaviour contracts of interaction state fixes.

use crate::support::mount_with;
use kivact::interaction::{TreeNode, use_tree};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions};
use kivact::view::{Key, RowHeight};

#[test]
fn a_tree_scrolls_through_its_caller_and_focuses_a_row_that_was_not_mounted() {
    let (screen, tree) = mount_with(
        RenderOptions::from_env().mirror(MirrorMode::Required),
        || {
            let keys: Vec<u32> = (0..300).collect();
            let nodes = signal(
                keys.iter()
                    .map(|key| TreeNode::new(*key, None, format!("row {key}")))
                    .collect::<Vec<_>>(),
            );
            let expanded = signal(Vec::<u32>::new());
            let selected = signal(Vec::<u32>::new());
            let target = signal(None::<u32>);
            let tree = use_tree(nodes, expanded, selected.into_selection())
                .page_size(5)
                .scroll_with(move |key| target.set(Some(key)));
            let rows = virtual_list(
                tree.rows(),
                |row: &kivact::interaction::TreeRow<u32>| row.key,
                move |row| {
                    let row = row.get();
                    let key = row.key;
                    let name = format!("row {key}");
                    (node()
                        .role(Role::TreeItem)
                        .name(name.clone())
                        .child(text(name))
                        .style(style().height(24)))
                    .part(tree.item(key))
                },
            )
            .row_height(RowHeight::Fixed(24.0))
            .overscan(0.0)
            .scroll_to(target)
            .test_id("rows")
            .style(style().height(120));
            (
                tree,
                rows.with_node(move |node| node.role(Role::Tree).part(tree.list())),
            )
        },
    );
    screen.get_by_role(Role::TreeItem, "row 0").click();
    assert!(screen.get_by_role(Role::TreeItem, "row 0").is_focused());
    screen.press_key(Key::End);
    screen.run_until_idle();
    assert_eq!(tree.active().get_untracked(), Some(299));
    assert!(
        screen.get_by_role(Role::TreeItem, "row 299").is_focused(),
        "the last row has the focus"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
