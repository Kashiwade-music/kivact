//! Role: Draw a themed tree using the shared row navigation and expansion parts.

use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::custom::Part;
use kivact_core::motion::{Transition, animated};
use kivact_core::state::{Binding, Value, effect, memo, signal};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::style::{Overflow, TransitionProp, style};
use kivact_core::view::{
    ExitLayout, HandlerResult, IntoView, ReadingDirection, Role, View, current_direction, for_each,
    node_ref, show,
};
use kivact_core::view::{Node, NodeBuilder, column, node, text};
use kivact_interaction::{ListSelection, Tree, TreeItemState, TreeNode, TreeRow, use_tree};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
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

type BranchRows<K> = HashMap<Option<K>, Vec<TreeRow<K>>>;
type RowNode<K> = Rc<dyn Fn(K, Value<TreeRow<K>>, Part<TreeItemState>) -> Node>;

/// Draw one clipped reveal per branch while keeping keyboard order on the
/// logical tree. The host retains removed branch paint without focus, hit
/// testing or accessibility. Reopening mounts fresh branch content.
#[track_caller]
pub fn tree_rows_with_reveal<K: Clone + Eq + Hash + 'static>(
    tree: Tree<K>,
    gap: f64,
    transition: Transition,
    render: impl Fn(K, Value<TreeRow<K>>, Part<TreeItemState>) -> Node + 'static,
) -> View {
    let visible = tree.rows();
    let branches = memo(move || {
        let mut result = BranchRows::new();
        let mut ancestors = Vec::new();
        for row in visible.get() {
            ancestors.truncate(row.level.saturating_sub(1) as usize);
            let parent = ancestors.last().cloned();
            ancestors.push(row.key.clone());
            result.entry(parent).or_default().push(row);
        }
        result
    });
    branch_rows(tree, branches, None, gap, transition, Rc::new(render))
}

fn branch_rows<K: Clone + Eq + Hash + 'static>(
    tree: Tree<K>,
    branches: Value<BranchRows<K>>,
    parent: Option<K>,
    gap: f64,
    transition: Transition,
    render: RowNode<K>,
) -> View {
    // A closing ancestor removes this host subtree as one group. Keep this
    // list's data until that unmount, so the host captures it as one exit.
    let previous = RefCell::new(Vec::new());
    let container = parent.clone();
    for_each(
        move || {
            if let Some(rows) = branches.with(|map| map.get(&parent).cloned()) {
                previous.replace(rows.clone());
                rows
            } else if parent.is_some() {
                previous.borrow().clone()
            } else {
                Vec::new()
            }
        },
        |row| (row.key.clone(), row.branch.has_children()),
        move |(key, has_children), row| {
            let part = tree.item_with_exit(key.clone());
            let line = render(key.clone(), row, part);
            if !has_children {
                return line;
            }
            let parent = Some(key);
            let lookup = parent.clone();
            let container = container.clone();
            let previous_open = Cell::new(false);
            let open = memo(move || {
                branches.with(|map| {
                    if map.contains_key(&container) {
                        previous_open.set(map.contains_key(&lookup));
                    }
                    previous_open.get()
                })
            });
            let progress = animated(
                move || if open.get() { 1.0f64 } else { 0.0 },
                transition.clone(),
            );
            let measured = node_ref();
            let rect = measured.rect();
            let last_height = signal(0.0f64);
            effect(move || {
                if open.get()
                    && let Some(rect) = rect.get()
                    && rect.height > 0.0
                {
                    last_height.set(rect.height);
                }
            });
            let render = render.clone();
            let exit_transition = transition.clone();
            let content = show(open, move || {
                let mut paint = style()
                    .gap(gap)
                    .padding_top(gap)
                    .shrink(0)
                    .exit(style().opacity(0.0))
                    .transition(TransitionProp::Paint, exit_transition.clone());
                if progress.get_untracked() < 1.0 {
                    paint = paint.enter(style().opacity(0.0));
                }
                column(branch_rows(
                    tree,
                    branches,
                    parent.clone(),
                    gap,
                    exit_transition.clone(),
                    render.clone(),
                ))
                .exit_layout(ExitLayout::Pop)
                .style(paint)
            });
            let shell = column(column(content).node_ref(measured).style(style().shrink(0))).style(
                move || {
                    let share = progress.get().clamp(0.0, 1.0);
                    if !open.get() || share < 1.0 {
                        style()
                            .height(last_height.get() * share)
                            .overflow(Overflow::Hidden)
                    } else {
                        style()
                    }
                },
            );
            column((line, shell))
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
