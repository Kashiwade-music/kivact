//! Role: Tests for tree view interactions and navigation.
//!
//! The unstyled `Tree`: the flattened rows with their level and position,
//! the keys of the ARIA tree view, activation, the accessibility actions,
//! the active row when a parent closes, the filter, single and multiple
//! selection, odd data, typeahead and the roving tab stop.

use std::time::Duration;

use crate::support::mount_with;
use kivact::interaction::Tree;
use kivact::prelude::*;
use kivact::test::{Element, RenderOptions, Screen, WindowMode};
use kivact::ui::TreeNode;
use kivact::view::{A11yAction, KeyCode, Modifiers};

type FileNode = TreeNode<&'static str>;

const SHIFT: Modifiers = Modifiers {
    shift: true,
    ctrl: false,
    alt: false,
    meta: false,
};
const CTRL: Modifiers = Modifiers {
    shift: false,
    ctrl: true,
    alt: false,
    meta: false,
};

struct Handles {
    expanded: Signal<Vec<&'static str>>,
    selected: Signal<Vec<&'static str>>,
    activated: Signal<Vec<&'static str>>,
    query: Signal<String>,
    readme: Signal<String>,
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
                activated,
                query,
                readme,
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

fn key_with(screen: &Screen, key: Key, modifiers: Modifiers) {
    screen.press_key_with(key, modifiers);
}

fn hold(screen: &Screen, code: KeyCode, modifiers: Modifiers) {
    screen.key_down(Key::Unidentified, code, modifiers);
}

fn release(screen: &Screen, code: KeyCode) {
    screen.key_up(Key::Unidentified, code, Modifiers::default());
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
fn a_label_follows_its_source() {
    let (screen, h) = mount_tree(Vec::new(), |tree| tree);
    h.readme.set("READ ME".to_string());
    screen.run_until_idle();
    assert!(item(&screen, "READ ME").exists());
    assert!(screen.query_by_role(Role::TreeItem, "README.md").is_none());
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

#[test]
fn star_opens_every_sibling() {
    let (screen, h) = mount_tree(Vec::new(), |tree| tree);
    focus_item(&screen, "src");
    screen.press_key(Key::Char('*'));
    assert_eq!(
        h.expanded.get_untracked(),
        ["src", "docs"],
        "the root parents; the leaf has nothing to open"
    );
    assert!(item(&screen, "src").is_focused());
    assert_eq!(h.selected.get_untracked(), ["src"], "the selection stays");
    focus_item(&screen, "lib.rs");
    screen.press_key(Key::Char('*'));
    assert_eq!(
        h.expanded.get_untracked(),
        ["src", "docs", "util"],
        "only the siblings of the row"
    );
    no_errors(&screen);
}

#[test]
fn enter_and_a_double_click_activate_and_a_single_click_does_not() {
    let (screen, h) = mount_tree(vec!["src"], |tree| tree);
    item(&screen, "lib.rs").click();
    assert!(
        h.activated.get_untracked().is_empty(),
        "a click only selects"
    );
    assert_eq!(h.selected.get_untracked(), ["lib"]);
    item(&screen, "main.rs").double_click();
    assert_eq!(
        h.activated.get_untracked(),
        ["main"],
        "once per double click"
    );
    screen.press_key(Key::Enter);
    assert_eq!(h.activated.get_untracked(), ["main", "main"]);
    focus_item(&screen, "util");
    screen.press_key(Key::Enter);
    assert_eq!(h.activated.get_untracked(), ["main", "main", "util"]);
    assert_eq!(
        h.expanded.get_untracked(),
        ["src"],
        "Enter does not open a parent"
    );
    item(&screen, "docs").double_click();
    assert_eq!(
        h.expanded.get_untracked(),
        ["src"],
        "nor does a double click"
    );
    assert_eq!(h.activated.get_untracked().last(), Some(&"docs"));
    no_errors(&screen);
}

#[test]
fn the_accessibility_actions_select_open_and_close_but_never_invoke() {
    let (screen, h) = mount_tree(vec!["src"], |tree| tree);
    let actions = item(&screen, "util").a11y_actions();
    assert!(actions.contains(&A11yAction::Select));
    assert!(actions.contains(&A11yAction::Expand));
    assert!(actions.contains(&A11yAction::Collapse));
    assert!(
        !actions.contains(&A11yAction::Activate),
        "Activate would run on every click"
    );
    let leaf = item(&screen, "main.rs").a11y_actions();
    assert!(leaf.contains(&A11yAction::Select));
    assert!(!leaf.contains(&A11yAction::Expand) && !leaf.contains(&A11yAction::Activate));
    item(&screen, "util").a11y(A11yAction::Expand);
    assert_eq!(h.expanded.get_untracked(), ["src", "util"]);
    assert_eq!(item(&screen, "util").is_expanded(), Some(true));
    item(&screen, "fmt.rs").a11y(A11yAction::Select);
    assert_eq!(h.selected.get_untracked(), ["fmt"]);
    assert!(item(&screen, "fmt.rs").is_selected());
    item(&screen, "util").a11y(A11yAction::Collapse);
    assert_eq!(h.expanded.get_untracked(), ["src"]);
    assert!(h.activated.get_untracked().is_empty());
    no_errors(&screen);
}

#[test]
fn the_active_row_moves_to_the_parent_that_closes_over_it() {
    let (screen, h) = mount_tree(vec!["src", "util"], |tree| tree);
    focus_item(&screen, "mod.rs");
    assert_eq!(h.tree.active().get_untracked(), Some("mod"));
    h.expanded.set(vec!["src"]);
    screen.run_until_idle();
    assert_eq!(
        h.tree.active().get_untracked(),
        Some("util"),
        "not the row at the old position"
    );
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(item(&screen, "util").is_focused());

    // Two levels at once: the nearest ancestor that is still shown.
    h.expanded.set(vec!["src", "util"]);
    screen.run_until_idle();
    focus_item(&screen, "fmt.rs");
    h.expanded.set(Vec::new());
    screen.run_until_idle();
    assert_eq!(h.tree.active().get_untracked(), Some("src"));
    no_errors(&screen);
}

#[test]
fn the_focus_follows_a_row_that_collapses_away_to_the_nearest_shown_ancestor() {
    // Closed by data.
    let (screen, h) = mount_tree(vec!["src", "util"], |tree| tree);
    focus_item(&screen, "mod.rs");
    h.expanded.set(vec!["src"]);
    screen.run_until_idle();
    assert!(item(&screen, "util").is_focused(), "not just the tab stop");
    assert_eq!(h.tree.active().get_untracked(), Some("util"));

    // Closed through the accessibility action on the grandparent.
    h.expanded.set(vec!["src", "util"]);
    screen.run_until_idle();
    focus_item(&screen, "fmt.rs");
    item(&screen, "src").a11y(A11yAction::Collapse);
    assert!(item(&screen, "src").is_focused());
    assert_eq!(h.selected.get_untracked(), ["fmt"], "the selection stays");

    // A row that is filtered away hands the focus to the active row, and a
    // focus elsewhere is left alone.
    h.expanded.set(vec!["src", "util"]);
    screen.run_until_idle();
    focus_item(&screen, "lib.rs");
    screen.get_by_role(Role::Button, "after").click();
    h.expanded.set(Vec::new());
    screen.run_until_idle();
    assert!(
        screen.get_by_role(Role::Button, "after").is_focused(),
        "the focus was not in the tree"
    );
    no_errors(&screen);
}

#[test]
fn a_filter_keeps_the_ancestors_shows_them_open_and_leaves_the_bindings_alone() {
    let (screen, h) = mount_tree(vec!["docs"], |tree| tree);
    h.selected.set(vec!["README"]);
    screen.run_until_idle();
    assert_eq!(shown(&h), ["src", "docs", "guide", "README"]);
    h.query.set("MOD".to_string());
    screen.run_until_idle();
    assert_eq!(
        shown(&h),
        ["src", "util", "mod"],
        "the match and its ancestors, ignoring case"
    );
    let rows = h.tree.rows().get_untracked();
    assert_eq!(
        rows.iter()
            .map(|r| (r.level, r.set_size, r.expanded))
            .collect::<Vec<_>>(),
        [(1, 1, true), (2, 1, true), (3, 1, false)],
        "shown open although the binding has them closed"
    );
    assert_eq!(item(&screen, "mod.rs").level(), Some(3));
    assert_eq!(
        h.expanded.get_untracked(),
        ["docs"],
        "the binding is not written"
    );
    assert_eq!(h.selected.get_untracked(), ["README"], "nor the selection");
    h.query.set("util".to_string());
    screen.run_until_idle();
    assert_eq!(
        shown(&h),
        ["src", "util"],
        "no children of a match come along"
    );
    h.query.set("nothing".to_string());
    screen.run_until_idle();
    assert!(shown(&h).is_empty());
    h.query.set(String::new());
    screen.run_until_idle();
    assert_eq!(shown(&h), ["src", "docs", "guide", "README"]);
    assert_eq!(h.expanded.get_untracked(), ["docs"]);
    assert_eq!(h.selected.get_untracked(), ["README"]);
    no_errors(&screen);
}

#[test]
fn a_multiple_selection_extends_in_the_order_of_the_shown_rows() {
    let (screen, h) = mount_tree(vec!["src"], |tree| tree);
    assert!(screen.get_by_role(Role::Tree, "Files").is_multiselectable());
    focus_item(&screen, "lib.rs");
    key_with(&screen, Key::ArrowDown, SHIFT);
    key_with(&screen, Key::ArrowDown, SHIFT);
    assert_eq!(
        h.selected.get_untracked(),
        ["lib", "util", "main"],
        "util is closed: its children are not in the range"
    );
    h.expanded.set(vec!["src", "util"]);
    screen.run_until_idle();
    focus_item(&screen, "lib.rs");
    hold(&screen, KeyCode::ShiftLeft, SHIFT);
    item(&screen, "main.rs").click();
    release(&screen, KeyCode::ShiftLeft);
    assert_eq!(
        h.selected.get_untracked(),
        ["lib", "util", "mod", "fmt", "main"],
        "the order on screen"
    );
    key_with(&screen, Key::ArrowUp, CTRL);
    key_with(&screen, Key::Space, CTRL);
    assert_eq!(
        h.selected.get_untracked(),
        ["lib", "util", "mod", "main"],
        "Ctrl+Space toggles"
    );
    key_with(&screen, Key::Char('a'), CTRL);
    assert_eq!(
        h.selected.get_untracked().len(),
        8,
        "every shown row: docs is closed, so guide is not selected"
    );
    no_errors(&screen);
}

#[test]
fn a_single_selection_replaces_for_every_modifier() {
    let (screen, h) = mount_nodes_kind(None, vec!["src"], true, |tree| tree);
    assert!(!screen.get_by_role(Role::Tree, "Files").is_multiselectable());
    focus_item(&screen, "lib.rs");
    key_with(&screen, Key::ArrowDown, SHIFT);
    assert_eq!(h.selected.get_untracked(), ["util"]);
    key_with(&screen, Key::ArrowDown, CTRL);
    assert_eq!(
        h.selected.get_untracked(),
        ["main"],
        "Ctrl moves and selects"
    );
    key_with(&screen, Key::Space, CTRL);
    assert_eq!(h.selected.get_untracked(), ["main"]);
    key_with(&screen, Key::Char('a'), CTRL);
    assert_eq!(h.selected.get_untracked(), ["main"], "no select all");
    hold(&screen, KeyCode::ShiftLeft, SHIFT);
    item(&screen, "lib.rs").click();
    release(&screen, KeyCode::ShiftLeft);
    assert_eq!(h.selected.get_untracked(), ["lib"], "no range");
    // Right goes to a row and selects only it.
    focus_item(&screen, "util");
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::ArrowRight);
    assert_eq!(h.selected.get_untracked(), ["mod"]);
    no_errors(&screen);
}

#[test]
fn a_cycle_a_missing_parent_and_a_repeated_key_are_handled_quietly() {
    let nodes = vec![
        TreeNode::new("a", Some("b"), "a"),
        TreeNode::new("b", Some("a"), "b"),
        TreeNode::new("c", Some("missing"), "c"),
        TreeNode::new("d", None, "d"),
        TreeNode::new("e", Some("a"), "e"),
        TreeNode::new("s", Some("s"), "s"),
        TreeNode::new("d", None, "d again"),
        TreeNode::new("f", Some("d"), "f"),
    ];
    let (screen, h) = mount_nodes(Some(nodes), vec!["d"], |tree| tree);
    assert_eq!(
        shown(&h),
        ["c", "d", "f"],
        "a cycle and what is below it are left out; the orphan is a root; \
         the first of two equal keys wins"
    );
    let rows = h.tree.rows().get_untracked();
    assert_eq!((rows[0].level, rows[0].set_size), (1, 2));
    assert_eq!((rows[2].level, rows[2].pos_in_set), (2, 1));
    focus_item(&screen, "c");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "f").is_focused());
    no_errors(&screen);
}

#[test]
fn a_disabled_node_is_shown_but_skipped() {
    let nodes = vec![
        TreeNode::new("a", None, "a"),
        TreeNode::new("b", None, "b").disabled(true),
        TreeNode::new("c", None, "c"),
    ];
    let (screen, h) = mount_nodes(Some(nodes), Vec::new(), |tree| tree);
    assert!(item(&screen, "b").is_disabled());
    focus_item(&screen, "a");
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "c").is_focused());
    assert_eq!(h.selected.get_untracked(), ["c"]);
    no_errors(&screen);
}

#[test]
fn typing_moves_to_the_next_shown_row_whose_label_starts_with_it() {
    let (screen, h) = mount_tree(vec!["src"], |tree| tree);
    focus_item(&screen, "src");
    screen.press_key(Key::Char('m'));
    assert!(item(&screen, "main.rs").is_focused());
    assert_eq!(h.selected.get_untracked(), ["main"]);
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('r'));
    assert!(item(&screen, "README.md").is_focused());
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('g'));
    assert!(
        item(&screen, "README.md").is_focused(),
        "guide.md is below a closed parent"
    );
    no_errors(&screen);
}

#[test]
fn the_tree_is_one_tab_stop() {
    let (screen, h) = mount_tree(vec!["src"], |tree| tree);
    assert_eq!(h.tree.active().get_untracked(), Some("src"));
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(item(&screen, "src").is_focused());
    screen.press_tab(false);
    assert!(
        screen.get_by_role(Role::Button, "after").is_focused(),
        "the other rows are no tab stops"
    );
    screen.press_tab(true);
    assert!(item(&screen, "src").is_focused());
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.get_by_role(Role::Button, "after").click();
    screen.press_tab(true);
    assert!(
        item(&screen, "util").is_focused(),
        "the last focused row is the tab stop"
    );
    no_errors(&screen);
}
