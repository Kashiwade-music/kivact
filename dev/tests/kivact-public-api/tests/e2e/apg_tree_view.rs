//! Role: Verify APG multi-select tree entry uses visible node order.
use crate::support::mount;
use kivact::interaction::{TreeNode, use_tree};
use kivact::prelude::*;

#[test]
fn tree_view_tab_enters_the_first_selected_node() {
    let (screen, selected) = mount(|| {
        let selected = signal(vec!["c", "b"]);
        let tree = use_tree(
            vec![
                TreeNode::new("a", None, "Alpha"),
                TreeNode::new("b", None, "Beta"),
                TreeNode::new("c", None, "Gamma"),
            ],
            signal(Vec::new()),
            selected,
        );
        (
            selected,
            column((
                node()
                    .role(Role::Button)
                    .name("Before")
                    .focusable(true)
                    .child(text("Before")),
                node().part(tree.list()).name("Tree").child(for_each(
                    tree.rows(),
                    |row| row.key,
                    move |row| {
                        let key = row.get().key;
                        node()
                            .part(tree.item(key))
                            .child(text(key))
                            .style(style().height(30))
                    },
                )),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::TreeItem, "Beta").is_focused());
    assert_eq!(selected.get_untracked(), vec!["c", "b"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
