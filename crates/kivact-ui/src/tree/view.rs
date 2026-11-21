//! Role: The tree component, its row rendering and its branch reveal.

use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::tree::look::TreeRowRecipeState;
use crate::tree::look::TreeTheme;
use kivact_core::custom::{Caller, Part};
use kivact_core::motion::Transition;
use kivact_core::state::{Callback, IntoValue, Signal, Value, signal};
use kivact_core::style::{TransitionProp, style};
use kivact_core::view::Variants;
use kivact_core::view::{
    IntoView, Orientation, ReadingDirection, Role, View, for_each, use_direction,
};
use kivact_core::view::{Node, column, node, text};
use kivact_interaction::{TreeApi, TreeItemState, TreeNode, TreeRow, use_tree};
use kivact_macros::component;
use std::rc::Rc;

/// A named tree with application-owned expansion and selection.
///
/// `label` is the tree's accessible name; it is not drawn. `items` are the
/// nodes, and `selected` is the selected key (or set of keys) that the tree
/// writes. `expanded` binds the expanded branches; without it the tree keeps
/// them itself. `header` places a header before the visible rows, and the
/// tree is named by it. `reveal` animates the branches with retained presence
/// (the space between rows is `reveal_gap`, default 0). `on_activate` runs when
/// a row is activated through Enter or a double click. `item` replaces a row's
/// body, and `marker_content` replaces the branch marker's body.
#[component]
pub fn tree<K: kivact_core::state::ItemKey, S: kivact_interaction::IntoSelection<K>>(
    label: Value<String>,
    items: Value<Vec<TreeNode<K>>>,
    selected: S,
    header: Option<View>,
    reveal: Option<Transition>,
    #[prop(default = 0.0)] reveal_gap: f64,
    expanded: Option<Signal<Vec<K>>>,
    on_activate: Option<Callback<K>>,
    item: Option<Rc<dyn Fn(Value<TreeRow<K>>, Value<TreeItemState>) -> View>>,
    marker_content: Option<Rc<dyn Fn(Value<TreeRowRecipeState>) -> View>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let expanded = expanded.unwrap_or_else(|| signal(Vec::new()));
    let mut behavior = use_tree(items, expanded, selected.into_selection());
    if let Some(activate) = on_activate {
        behavior = behavior.on_activate(move |key| activate.call(key));
    }
    let list = behavior.list();
    let state = list.state();
    let rows = render_rows(behavior, variants.clone(), item, marker_content);
    let rows = match reveal {
        Some(transition) => rows.reveal(transition).reveal_gap(reveal_gap),
        None => rows,
    };
    let header_ref = kivact_core::view::node_ref();
    let mut root = node().name(label);
    if header.is_some() {
        root = root.labelled_by(header_ref);
    }
    let header = header.map(|content| node().node_ref(header_ref).child(content));
    with_base_recipe_style(
        root.part(list),
        |r: &TreeTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    )
    .child((header, rows))
}

/// Visible rows for a custom tree container, from `tree` and the rows' render.
///
/// Attach the part `render` gets to each row. `presentation` is an additional
/// key for changes that require rebuilding the row; use `|_| ()` for a row
/// whose presentation stays reactive. `reveal` animates each branch with the
/// transition, and `reveal_gap` is the space between the revealed rows
/// (default 0).
#[component]
pub fn tree_rows<K: kivact_core::state::ItemKey, P: kivact_core::state::ItemKey>(
    tree: TreeApi<K>,
    presentation: Rc<dyn Fn(&TreeRow<K>) -> P>,
    render: Rc<dyn Fn(K, Value<TreeRow<K>>, Part<TreeItemState>) -> Node>,
    reveal: Option<Transition>,
    #[prop(default = 0.0)] reveal_gap: f64,
) -> impl IntoView {
    match reveal {
        Some(transition) => revealed_rows(tree, reveal_gap, transition, render),
        None => {
            let tree_for_rows = tree;
            for_each(
                tree.rows(),
                move |row| {
                    (
                        row.key.clone(),
                        row.branch.has_children(),
                        presentation(row),
                    )
                },
                move |row| {
                    let key = row.with_untracked(|row| row.key.clone());
                    render(key.clone(), row, tree_for_rows.item(key))
                },
            )
            .into_view()
        }
    }
}

type RowNode<K> = Rc<dyn Fn(K, Value<TreeRow<K>>, Part<TreeItemState>) -> Node>;

type RowContent<K> = Rc<dyn Fn(Value<TreeRow<K>>, Value<TreeItemState>) -> View>;
type MarkerContent = Rc<dyn Fn(Value<TreeRowRecipeState>) -> View>;

/// Removed rows leave navigation immediately; engine size presence retains
/// their paint and releases their layout over the supplied transition.
fn revealed_rows<K: kivact_core::state::ItemKey>(
    tree: TreeApi<K>,
    gap: f64,
    transition: Transition,
    render: RowNode<K>,
) -> View {
    let host = kivact_core::view::node_ref();
    let presence = host.presence();
    let rows = for_each(
        tree.rows(),
        |row| (row.key.clone(), row.branch.has_children()),
        move |row| {
            let key = row.with_untracked(|row| row.key.clone());
            let part = tree.item_with_exit(key.clone());
            // Existing rows are fully laid out on the tree's first frame.
            // Rows added to a mounted tree reveal through engine presence.
            let look = style()
                .flex_shrink(0)
                .exit_size(Orientation::Vertical)
                .transition(TransitionProp::Size, transition.clone());
            let look = if presence.get_untracked() == kivact_core::view::Presence::Present {
                look.enter_size(Orientation::Vertical)
            } else {
                look
            };
            column(render(key, row, part)).style(look)
        },
    );
    column(rows)
        .node_ref(host)
        .style(style().gap(gap))
        .into_view()
}

fn row_marker<K: kivact_core::state::ItemKey>(
    behavior: TreeApi<K>,
    key: K,
    row: Value<TreeRow<K>>,
    state: Value<TreeItemState>,
    recipe_state: Value<TreeRowRecipeState>,
    row_variants: &Variants,
    marker_content: Option<&MarkerContent>,
) -> Node {
    let rtl = use_direction() == ReadingDirection::Rtl;
    let parent = row.with_untracked(|row| row.branch.has_children());
    let marker = with_recipe_style(
        node(),
        |r: &TreeTheme| r.marker_recipe(),
        row_variants,
        move || recipe_state.get(),
    );
    let marker = marker.hidden(move || !recipe_state.get().has_branches);
    if parent {
        marker
            .role(Role::Button)
            .name(kivact_core::i18n::message(
                "tree.toggle_children",
                "Toggle children",
            ))
            .expanded(move || state.get().branch.is_expanded())
            .disabled(move || state.get().input.is_disabled())
            .on_press(move || behavior.toggle(key.clone()))
            .child(match marker_content {
                Some(content) => content(recipe_state.into_value()),
                None => text(move || {
                    if state.get().branch.is_expanded() {
                        "⌄"
                    } else if rtl {
                        "‹"
                    } else {
                        "›"
                    }
                })
                .into_view(),
            })
    } else {
        marker
    }
}

fn row_recipe<K: kivact_core::state::ItemKey>(
    recipe_key: K,
    row: Value<TreeRow<K>>,
    state: Value<TreeItemState>,
    ancestors: Value<std::collections::HashMap<K, Vec<bool>>>,
    has_branches: Value<bool>,
    custom_marker: bool,
) -> Value<TreeRowRecipeState> {
    kivact_core::state::memo(move || TreeRowRecipeState {
        item: state.get(),
        ancestor_last: ancestors.with(|rows| rows.get(&recipe_key).cloned().unwrap_or_default()),
        is_last: row.with(TreeRow::is_last),
        has_branches: has_branches.get(),
        custom_marker,
    })
    .into_value()
}

fn render_rows<K: kivact_core::state::ItemKey>(
    behavior: TreeApi<K>,
    variants: Variants,
    item: Option<RowContent<K>>,
    marker_content: Option<MarkerContent>,
) -> crate::tree::TreeRows<K, ()> {
    let row_variants = variants;
    let ancestors = behavior.ancestor_last();
    let has_branches = kivact_core::state::memo(move || {
        behavior
            .rows()
            .with(|rows| rows.iter().any(|row| row.branch.has_children()))
    });

    tree_rows(
        behavior,
        |_| (),
        move |key, row, part| {
            let state = part.state();
            let recipe_state = row_recipe(
                key.clone(),
                row,
                state,
                ancestors,
                has_branches.into_value(),
                marker_content.is_some(),
            );
            let label = behavior.label(key.clone());
            let marker = row_marker(
                behavior,
                key,
                row,
                state,
                recipe_state.into_value(),
                &row_variants,
                marker_content.as_ref(),
            );
            let content = match &item {
                Some(item) => item(row, state),
                None => text(label).into_view(),
            };
            with_recipe_style(
                node().part(part),
                |r: &TreeTheme| r.item_recipe(),
                &row_variants,
                move || recipe_state.get(),
            )
            .child((marker, content))
        },
    )
}
