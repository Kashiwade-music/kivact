//! Role: Tests for tree view interactions and navigation.
//!
//! The unstyled `Tree`: the flattened rows with their level and position,
//! the keys of the ARIA tree view, activation, the accessibility actions,
//! the active row when a parent closes, the filter, single and multiple
//! selection, odd data, typeahead and the roving tab stop.

use crate::support::mount_with;
use kivact::interaction::Tree;
use kivact::prelude::*;
use kivact::test::{Element, RenderOptions, Screen, WindowMode};
use kivact::ui::TreeNode;

type FileNode = TreeNode<&'static str>;

struct Handles {
    expanded: Signal<Vec<&'static str>>,
    selected: Signal<Vec<&'static str>>,
    tree: Tree<&'static str>,
}

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
}

/// src (lib.rs, util (mod.rs, fmt.rs), main.rs), docs (guide.md), README.md
fn files(readme: Signal<String>) -> Vec<FileNode> {
    vec![
        TreeNode::new("src", None, "src"),
        TreeNode::new("lib", Some("src"), "lib.rs"),
        TreeNode::new("util", Some("src"), "util"),
        TreeNode::new("mod", Some("util"), "mod.rs"),
        TreeNode::new("fmt", Some("util"), "fmt.rs"),
        TreeNode::new("main", Some("src"), "main.rs"),
        TreeNode::new("docs", None, "docs"),
        TreeNode::new("guide", Some("docs"), "guide.md"),
        TreeNode::new("README", None, move || readme.get()),
    ]
}

fn mount_tree(
    expanded: Vec<&'static str>,
    configure: impl FnOnce(Tree<&'static str>) -> Tree<&'static str> + 'static,
) -> (Screen, Handles) {
    mount_nodes(None, expanded, configure)
}

fn mount_nodes(
    custom: Option<Vec<FileNode>>,
    expanded: Vec<&'static str>,
    configure: impl FnOnce(Tree<&'static str>) -> Tree<&'static str> + 'static,
) -> (Screen, Handles) {
    mount_nodes_kind(custom, expanded, false, configure)
}

fn mount_nodes_kind(
    custom: Option<Vec<FileNode>>,
    expanded: Vec<&'static str>,
    single: bool,
    configure: impl FnOnce(Tree<&'static str>) -> Tree<&'static str> + 'static,
) -> (Screen, Handles) {
    mount_with(real_window(), move || {
        let readme = signal("README.md".to_string());
        let nodes = signal(custom.unwrap_or_else(|| files(readme)));
        let expanded = signal(expanded);
        let selected = signal(Vec::<&'static str>::new());
        let selection = if single {
            kivact::interaction::ListSelection::Single(kivact::binding(
                memo(move || selected.with(|keys| keys.first().copied())),
                move |key: Option<&'static str>| selected.set(key.into_iter().collect()),
            ))
        } else {
            kivact::interaction::ListSelection::Multiple(selected.into())
        };

        let activated = signal(Vec::<&'static str>::new());
        let query = signal(String::new());
        let tree = configure(
            kivact::interaction::use_tree(nodes, expanded, selection)
                .filter(query)
                .on_activate(move |key| {
                    activated.update(|keys| keys.push(key));
                }),
        );
        let rows = for_each(
            tree.rows(),
            |row| (row.key, row.has_children),
            move |_, row| {
                let row = row.get_untracked();
                let label = move || {
                    nodes.with(|nodes| {
                        nodes
                            .iter()
                            .find(|node| *node.key() == row.key)
                            .map(|node| node.label())
                            .unwrap_or_default()
                    })
                };
                (node()
                    .child(text(label))
                    .style(style().height(24).padding_left(row.level as i32 * 12)))
                .part(tree.item(row.key))
            },
        );
        let view = column((
            button("before"),
            (column(rows).name("Files").test_id("tree")).part(tree.list()),
            button("after"),
        ));
        (
            Handles {
                expanded,
                selected,
                tree,
            },
            view,
        )
    })
}

fn button(name: &'static str) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .child(text(name))
        .style(style().size(120, 28))
}

fn item<'a>(screen: &'a Screen, name: &str) -> Element<'a> {
    screen.get_by_role(Role::TreeItem, name)
}

fn focus_item(screen: &Screen, name: &str) {
    item(screen, name).click();
    assert!(item(screen, name).is_focused());
}

fn no_errors(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn shown(h: &Handles) -> Vec<&'static str> {
    h.tree
        .rows()
        .get_untracked()
        .iter()
        .map(|row| row.key)
        .collect()
}

#[test]
fn the_rows_are_flattened_in_tree_order_with_their_level_and_position() {
    let (screen, h) = mount_tree(vec!["src"], |tree| tree);
    assert_eq!(
        shown(&h),
        ["src", "lib", "util", "main", "docs", "README"],
        "depth first, the children of a closed parent left out"
    );
    let rows = h.tree.rows().get_untracked();
    let facts: Vec<_> = rows
        .iter()
        .map(|r| {
            (
                r.level,
                r.set_size,
                r.pos_in_set,
                r.has_children,
                r.expanded,
            )
        })
        .collect();
    assert_eq!(
        facts,
        [
            (1, 3, 1, true, true),
            (2, 3, 1, false, false),
            (2, 3, 2, true, false),
            (2, 3, 3, false, false),
            (1, 3, 2, true, false),
            (1, 3, 3, false, false),
        ]
    );
    let tree = screen.get_by_role(Role::Tree, "Files");
    assert!(tree.is_multiselectable());
    for (name, level, size, pos, expanded) in [
        ("src", 1, 3, 1, Some(true)),
        ("lib.rs", 2, 3, 1, None),
        ("util", 2, 3, 2, Some(false)),
        ("main.rs", 2, 3, 3, None),
        ("docs", 1, 3, 2, Some(false)),
        ("README.md", 1, 3, 3, None),
    ] {
        let it = item(&screen, name);
        assert_eq!(
            (it.level(), it.set_size(), it.pos_in_set(), it.is_expanded()),
            (Some(level), Some(size), Some(pos), expanded),
            "{name}"
        );
    }
    h.expanded.set(vec!["src", "util"]);
    screen.run_until_idle();
    assert_eq!(
        shown(&h),
        ["src", "lib", "util", "mod", "fmt", "main", "docs", "README"]
    );
    let fmt = item(&screen, "fmt.rs");
    assert_eq!(
        (fmt.level(), fmt.pos_in_set(), fmt.set_size()),
        (Some(3), Some(2), Some(2))
    );
    assert_eq!(item(&screen, "util").is_expanded(), Some(true));
    no_errors(&screen);
}

#[test]
fn right_opens_a_closed_parent_then_goes_to_its_first_child() {
    let (screen, h) = mount_tree(vec!["src"], |tree| tree);
    focus_item(&screen, "util");
    screen.press_key(Key::ArrowRight);
    assert_eq!(h.expanded.get_untracked(), ["src", "util"]);
    assert!(
        item(&screen, "util").is_focused(),
        "opening keeps the focus"
    );
    assert_eq!(h.selected.get_untracked(), ["util"]);
    screen.press_key(Key::ArrowRight);
    assert!(item(&screen, "mod.rs").is_focused(), "the first child");
    assert_eq!(h.selected.get_untracked(), ["mod"], "and selects only it");
    screen.press_key(Key::ArrowRight);
    assert!(item(&screen, "mod.rs").is_focused(), "a leaf: nothing");
    assert_eq!(h.expanded.get_untracked(), ["src", "util"]);
    no_errors(&screen);
}

#[test]
fn left_closes_an_open_parent_then_goes_to_the_parent() {
    let (screen, h) = mount_tree(vec!["src", "util"], |tree| tree);
    focus_item(&screen, "fmt.rs");
    screen.press_key(Key::ArrowLeft);
    assert!(
        item(&screen, "util").is_focused(),
        "a leaf goes to its parent"
    );
    assert_eq!(h.selected.get_untracked(), ["util"]);
    screen.press_key(Key::ArrowLeft);
    assert_eq!(h.expanded.get_untracked(), ["src"], "an open parent closes");
    assert!(item(&screen, "util").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(
        item(&screen, "src").is_focused(),
        "a closed parent goes to its parent"
    );
    screen.press_key(Key::ArrowLeft);
    assert!(
        h.expanded.get_untracked().is_empty(),
        "the open root closes"
    );
    screen.press_key(Key::ArrowLeft);
    assert!(item(&screen, "src").is_focused(), "a closed root: nothing");
    focus_item(&screen, "README.md");
    screen.press_key(Key::ArrowLeft);
    assert!(
        item(&screen, "README.md").is_focused(),
        "a root leaf: nothing"
    );
    assert_eq!(h.selected.get_untracked(), ["README"]);
    no_errors(&screen);
}
