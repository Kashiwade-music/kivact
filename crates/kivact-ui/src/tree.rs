//! Role: Draw a themed tree using the shared row navigation and expansion parts.

use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::control::{Node, NodeBuilder, node, text};
use kivact_core::primitive::Part;
use kivact_core::style::style;
use kivact_core::view::{
    HandlerResult, IntoText, IntoValue, IntoView, ReadingDirection, Role, View, current_direction,
    for_each,
};
use kivact_core::{Binding, Value};
use kivact_interaction::{ListSelection, Tree, TreeItemState, TreeNode, TreeRow, use_tree};
use std::hash::Hash;
use std::rc::Rc;

type RowContent<K> = Rc<dyn Fn(Value<TreeRow<K>>, Value<TreeItemState>) -> View>;

/// A keyed tree view drawn with the current theme's tree recipes.
pub struct TreeView<K: Clone + Eq + Hash + 'static> {
    root: Node,
    behavior: Tree<K>,
    item: Option<RowContent<K>>,
    variants: Variants,
}

/// A named tree with application-owned expansion and selection.
#[track_caller]
pub fn tree<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    nodes: impl IntoValue<Vec<TreeNode<K>>>,
    expanded: impl Into<Binding<Vec<K>>>,
    selection: ListSelection<K>,
) -> TreeView<K> {
    TreeView {
        root: node().name(label),
        behavior: use_tree(nodes, expanded, selection),
        item: None,
        variants: Variants::default(),
    }
}

impl<K: Clone + Eq + Hash + 'static> TreeView<K> {
    /// Call `activate` when a row is activated through Enter or double click.
    pub fn on_activate<R: HandlerResult>(mut self, activate: impl Fn(K) -> R + 'static) -> Self {
        self.behavior = self.behavior.on_activate(activate);
        self
    }

    /// Use a custom row body in place of the row's default label.
    pub fn item<V: IntoView>(
        mut self,
        item: impl Fn(Value<TreeRow<K>>, Value<TreeItemState>) -> V + 'static,
    ) -> Self {
        self.item = Some(Rc::new(move |row, state| item(row, state).into_view()));
        self
    }

    /// Set a theme-specific variant for the tree's recipes.
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for TreeView<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// Build visible rows for a custom tree container and `Tree::list()`.
/// Attach the supplied part to the row. `presentation` is an additional
/// key for changes that require rebuilding the row. Use `|_| ()` for a
/// row whose presentation stays reactive.
#[track_caller]
pub fn tree_rows<K: Clone + Eq + Hash + 'static, P: Clone + Eq + Hash + 'static>(
    tree: Tree<K>,
    presentation: impl Fn(&TreeRow<K>) -> P + 'static,
    render: impl Fn(K, Value<TreeRow<K>>, Part<TreeItemState>) -> Node + 'static,
) -> View {
    for_each(
        tree.rows(),
        move |row| {
            (
                row.key.clone(),
                row.branch.has_children(),
                presentation(row),
            )
        },
        move |(key, _, _), row| {
            let part = tree.item(key.clone());
            render(key, row, part)
        },
    )
    .into_view()
}

impl<K: Clone + Eq + Hash + 'static> IntoView for TreeView<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            behavior,
            item,
            variants,
        } = self;
        let rtl = current_direction() == ReadingDirection::Rtl;
        let list = behavior.list();
        let state = list.state();
        let row_variants = variants.clone();
        let rows = tree_rows(
            behavior,
            |_| (),
            move |key, row, part| {
                let state = part.state();
                let parent = row.with_untracked(|row| row.branch.has_children());
                let label = behavior.label(key.clone());
                let marker = node().style(style().width(16).shrink(0));
                let marker = if parent {
                    marker
                        .role(Role::Generic)
                        .disabled(move || state.get().input.disabled)
                        .on_press(move || behavior.toggle(key.clone()))
                        .child(text(move || {
                            if state.get().branch.is_expanded() {
                                "⌄"
                            } else if rtl {
                                "‹"
                            } else {
                                "›"
                            }
                        }))
                } else {
                    marker
                };
                let content = item
                    .as_ref()
                    .map_or_else(|| text(label).into_view(), |item| item(row, state));
                with_recipe_style(
                    node().part(part),
                    |r| r.tree_recipes().item_recipe(),
                    &row_variants,
                    move || state.get(),
                )
                .child((marker, content))
            },
        );
        with_base_recipe_style(
            root.part(list),
            |r| r.tree_recipes().root_recipe(),
            &variants,
            move || state.get(),
        )
        .child(rows)
        .into_view()
    }
}
