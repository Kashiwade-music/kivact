//! Role: Draw a themed tree using the shared row navigation and expansion parts.

use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::custom::Part;
use kivact_core::motion::Transition;
use kivact_core::state::{Binding, Value, signal};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::style::{TransitionProp, style};
use kivact_core::view::{
    HandlerResult, IntoView, Orientation, ReadingDirection, Role, View, for_each, use_direction,
};
use kivact_core::view::{Node, NodeBuilder, column, node, text};
use kivact_interaction::{Selection, TreeApi, TreeItemState, TreeNode, TreeRow, use_tree};
use std::rc::Rc;

type RowContent<K> = kivact_core::view::ViewFn<(Value<TreeRow<K>>, Value<TreeItemState>)>;

/// A keyed tree view drawn with the current theme's tree recipes.
pub struct Tree<K: kivact_core::state::ItemKey> {
    root: Node,
    nodes: Value<Vec<TreeNode<K>>>,
    selection: Selection<K>,
    expanded: Binding<Vec<K>>,
    // xtask-allow(state-fields): Optional custom row content chosen during construction.
    item: Option<RowContent<K>>,
    variants: Variants,
    // xtask-allow(state-fields): Optional construction callback; navigation transitions belong to use_tree.
    activate: Option<kivact_core::state::Callback<K>>,
    // xtask-allow(state-fields): Optional construction slot independent of tree navigation.
    header: Option<View>,
    // xtask-allow(state-fields): Optional construction transition independent of expansion bindings.
    reveal: Option<(f64, Transition)>,
}

/// A named tree with application-owned expansion and selection.
#[track_caller]
pub fn tree<K: kivact_core::state::ItemKey>(
    label: impl IntoText,
    nodes: impl IntoValue<Vec<TreeNode<K>>>,
) -> Tree<K> {
    Tree {
        root: node().name(label),
        nodes: nodes.into_value(),
        selection: kivact_interaction::IntoSelection::into_selection(signal(None)),
        expanded: kivact_core::state::IntoBinding::into_binding(signal(Vec::new())),
        item: None,
        variants: Variants::default(),
        activate: None,
        header: None,
        reveal: None,
    }
}

impl<K: kivact_core::state::ItemKey> Tree<K> {
    /// Place a header before the visible rows.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }
    /// Reveal branches with retained presence, as in `tree_rows`.
    pub fn reveal(mut self, gap: impl Into<f64>, transition: Transition) -> Self {
        self.reveal = Some((gap.into(), transition));
        self
    }
    /// Bind the selected keys.
    pub fn selection(mut self, selection: impl kivact_interaction::IntoSelection<K>) -> Self {
        self.selection = selection.into_selection();
        self
    }
    /// Bind expanded branches.
    pub fn expanded(mut self, expanded: impl kivact_core::state::IntoBinding<Vec<K>>) -> Self {
        self.expanded = expanded.into_binding();
        self
    }
    /// Call `activate` when a row is activated through Enter or double click.
    pub fn on_activate<R: HandlerResult>(mut self, activate: impl Fn(K) -> R + 'static) -> Self {
        self.activate = Some(kivact_core::state::callback(activate));
        self
    }

    /// Use a custom row body in place of the row's default label.
    pub fn item<V: IntoView>(
        mut self,
        item: impl Fn(Value<TreeRow<K>>, Value<TreeItemState>) -> V + 'static,
    ) -> Self {
        self.item = Some(kivact_core::view::ViewFn::from(move |(row, state)| {
            item(row, state)
        }));
        self
    }

    /// Set a theme-specific variant for the tree's recipes.
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}

impl<K: kivact_core::state::ItemKey> NodeBuilder for Tree<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// Build visible rows for a custom tree container and `Tree::list()`.
/// Attach the supplied part to the row. `presentation` is an additional
/// key for changes that require rebuilding the row. Use `|_| ()` for a
/// row whose presentation stays reactive.
#[track_caller]
pub fn tree_rows<K: kivact_core::state::ItemKey, P: kivact_core::state::ItemKey>(
    tree: TreeApi<K>,
    presentation: impl Fn(&TreeRow<K>) -> P + 'static,
    render: impl Fn(K, Value<TreeRow<K>>, Part<TreeItemState>) -> Node + 'static,
) -> TreeRows<K> {
    let tree_for_rows = tree;
    let render: RowNode<K> = Rc::new(render);
    let ordinary = render.clone();
    let view = for_each(
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
            ordinary(key.clone(), row, tree_for_rows.item(key))
        },
    )
    .into_view();
    TreeRows {
        tree,
        view,
        render,
        reveal: None,
    }
}

/// Visible tree rows with optional animated branch reveals.
pub struct TreeRows<K: kivact_core::state::ItemKey> {
    tree: TreeApi<K>,
    view: View,
    render: RowNode<K>,
    reveal: Option<(f64, Transition)>,
}
impl<K: kivact_core::state::ItemKey> TreeRows<K> {
    /// Reveal each branch with the supplied spacing and transition.
    pub fn reveal(mut self, gap: impl Into<f64>, transition: Transition) -> Self {
        self.reveal = Some((gap.into(), transition));
        self
    }
}
impl<K: kivact_core::state::ItemKey> IntoView for TreeRows<K> {
    fn into_view(self) -> View {
        match self.reveal {
            Some((gap, transition)) => revealed_rows(self.tree, gap, transition, self.render),
            None => self.view,
        }
    }
}

type RowNode<K> = Rc<dyn Fn(K, Value<TreeRow<K>>, Part<TreeItemState>) -> Node>;

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
                .shrink(0)
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

impl<K: kivact_core::state::ItemKey> IntoView for Tree<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            nodes,
            selection,
            expanded,
            activate,
            item,
            variants,
            header,
            reveal,
        } = self;
        let mut behavior = use_tree(nodes, expanded, selection);
        if let Some(activate) = activate {
            behavior = behavior.on_activate(move |key| activate.call(key));
        }
        let rtl = use_direction().get() == ReadingDirection::Rtl;
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
                let marker = with_recipe_style(
                    node(),
                    |r| r.tree_recipes().marker_recipe(),
                    &row_variants,
                    move || state.get(),
                );
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
                    .map_or_else(|| text(label).into_view(), |item| item.call((row, state)));
                with_recipe_style(
                    node().part(part),
                    |r| r.tree_recipes().item_recipe(),
                    &row_variants,
                    move || state.get(),
                )
                .child((marker, content))
            },
        );
        let rows = match reveal {
            Some((gap, transition)) => rows.reveal(gap, transition),
            None => rows,
        };
        with_base_recipe_style(
            root.part(list),
            |r| r.tree_recipes().root_recipe(),
            &variants,
            move || state.get(),
        )
        .child((header, rows))
        .into_view()
    }
}
