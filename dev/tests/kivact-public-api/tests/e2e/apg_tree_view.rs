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

#[test]
fn ui_tree_exposes_parent_expansion_and_leaf_hierarchy() {
    let (screen, expanded) = mount(|| {
        let expanded = signal(vec!["parent"]);
        (
            expanded,
            kivact::ui::tree(
                "Files",
                vec![
                    TreeNode::new("parent", None, "Parent"),
                    TreeNode::new("child", Some("parent"), "Child"),
                    TreeNode::new("leaf", None, "Leaf"),
                ],
            )
            .expanded(expanded)
            .header(text("Files header")),
        )
    });
    let tree = screen.within(screen.get_by_role(Role::Tree, "Files"));
    let parent = tree.get_by_role(Role::TreeItem, "Parent");
    assert_eq!(parent.is_expanded(), Some(true));
    assert_eq!(parent.level(), Some(1));
    assert_eq!(parent.set_size(), Some(2));
    assert_eq!(parent.pos_in_set(), Some(1));
    let child = tree.get_by_role(Role::TreeItem, "Child");
    assert_eq!(child.level(), Some(2));
    assert_eq!(child.set_size(), Some(1));
    assert_eq!(child.pos_in_set(), Some(1));
    assert_eq!(child.is_expanded(), None);
    let leaf = tree.get_by_role(Role::TreeItem, "Leaf");
    assert_eq!(leaf.is_expanded(), None);
    assert!(
        !leaf
            .a11y_actions()
            .contains(&kivact::view::A11yAction::Expand)
    );
    parent.a11y(kivact::view::A11yAction::Collapse);
    assert_eq!(expanded.get_untracked(), Vec::<&str>::new());
    assert_eq!(parent.is_expanded(), Some(false));
    assert!(tree.query_by_role(Role::TreeItem, "Child").is_none());
    parent.a11y(kivact::view::A11yAction::Expand);
    assert_eq!(tree.get_by_role(Role::TreeItem, "Child").level(), Some(2));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn ui_tree_keeps_selection_separate_from_roving_focus() {
    let (screen, selected) = mount(|| {
        let selected = signal(vec!["a"]);
        (
            selected,
            kivact::ui::tree(
                "Choices",
                vec![
                    TreeNode::new("a", None, "Alpha"),
                    TreeNode::new("b", None, "Beta"),
                ],
            )
            .selection(selected),
        )
    });
    let alpha = screen.get_by_role(Role::TreeItem, "Alpha");
    alpha.focus();
    let control = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    alpha.press_key_with(Key::ArrowDown, control);
    assert!(screen.get_by_role(Role::TreeItem, "Beta").is_focused());
    assert_eq!(selected.get_untracked(), vec!["a"]);
    screen
        .get_by_role(Role::TreeItem, "Beta")
        .press_key_with(Key::Space, control);
    assert_eq!(selected.get_untracked(), vec!["a", "b"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
