//! Role: TreeApi navigation with expansion and selection.
//!
//! Unstyled behavior and semantics of a tree view: a flattened, expandable,
//! selectable list of rows with the keys of the ARIA authoring practices.
//!
//! [`use_tree`] derives the shown rows from the [`TreeNode`] parents, the
//! expanded keys (`Binding<Vec<K>>`) and the selection, and runs them through
//! one [`ListApi`]: its registry, selection and navigation. The
//! rows are metadata ([`TreeRow`]); `list` and `item` are the `TreeApi` and
//! `TreeItem` parts, with [`TreeItemState`] (`input`, `selected`, `active`,
//! `branch`, `level`). Only a parent has Expand/Collapse, so a row's part
//! is built for its `(key, has_children)` identity while the other metadata stays reactive. The tree adds no
//! `scroll_with` or `page_size` of its own; single selection is
//! `Selection::Single`. The Creative outliner is built on a
//! `TreeApi<u64>`.

use kivact_core::custom::Reactive;
use kivact_core::view::HandlerResult;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

use kivact_core::custom::{Owned, Owner, Part, PartState};

use super::press::is_double_press;
use super::{ListApi, ListItem, ListOptions, ListRangeOptions, use_list};
use kivact_core::state::{Binding, Callback, Signal, Value, memo, signal};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::view::A11yAction;
use kivact_core::view::{Handled, Key, KeyEvent, PointerEvent, Role};

/// One node of the data a [`TreeApi`] shows: its key, the key of its parent,
/// its label and whether it is disabled.
///
/// A tree is described by a flat list of nodes, so the application's data can
/// stay flat and a node moves by changing its `parent`. The label is a
/// [`Reactive`], like that of [`ListItem`]: a
/// constant, a closure, a `Signal` or a `message(..)` value.
#[derive(Clone, PartialEq)]
pub struct TreeNode<K> {
    pub(crate) key: K,
    /// The parent's key. `None`, and a key that is not among the nodes, make
    /// the node a root.
    pub(crate) parent: Option<K>,
    pub(crate) label: Reactive<String>,
    pub(crate) disabled: bool,
}

impl<K> TreeNode<K> {
    /// A node whose children are the nodes that name it as their `parent`.
    pub fn new(key: K, parent: Option<K>, label: impl IntoText) -> Self {
        Self {
            key,
            parent,
            label: Reactive::new(label.into_text()),
            disabled: false,
        }
    }

    /// A disabled node is shown, but the keys skip it and it cannot be
    /// selected.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The application's identity of the node.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// The parent's key; `None` for a root.
    pub fn parent(&self) -> Option<&K> {
        self.parent.as_ref()
    }

    /// The label's current text; tracked when read inside a computation.
    #[track_caller]
    pub fn label(&self) -> String {
        self.label.get()
    }

    /// Whether the node is [`disabled`](Self::disabled).
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

/// Whether a tree row is a parent and whether its children are shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreeBranch {
    /// The row has no children (the ones that match, while a filter is set).
    Leaf,
    /// The row has children and they are hidden.
    Collapsed,
    /// The row has children and they are shown.
    Expanded,
}

impl TreeBranch {
    /// Whether the row is a parent.
    pub fn has_children(self) -> bool {
        self != Self::Leaf
    }

    /// Whether the children are shown.
    pub fn is_expanded(self) -> bool {
        self == Self::Expanded
    }
}

/// A row of the flattened tree: what a kit needs to draw it.
///
/// Key `for_each` rows by `(key, branch.has_children())` so a parent/leaf
/// transition rebuilds the row Part and its accessibility action set.
#[derive(Clone, PartialEq, Debug)]
pub struct TreeRow<K> {
    pub key: K,
    /// The depth, counted from 1 for a root.
    pub level: u32,
    /// How many siblings the row has, itself included.
    pub set_size: u32,
    /// The position among the siblings, counted from 1.
    pub pos_in_set: u32,
    /// Whether the node has children (the ones that match, while a filter is
    /// set) and whether they are shown.
    pub branch: TreeBranch,
}

impl<K> TreeRow<K> {
    /// Whether this is the last visible sibling.
    pub fn is_last(&self) -> bool {
        self.pos_in_set == self.set_size
    }
}

/// A shown row with the parts of its node that a row reads.
#[derive(Clone, PartialEq)]
struct Entry<K> {
    row: TreeRow<K>,
    label: String,
    disabled: bool,
}

/// The shape of the nodes, by position among the distinct keys.
struct Forest<K> {
    keys: Vec<K>,
    /// Where each node is in the list the forest was built from.
    src: Vec<usize>,
    parent: Vec<Option<usize>>,
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
    /// The nodes reachable from a root, parents before children, siblings in
    /// list order.
    order: Vec<usize>,
    index: HashMap<K, usize>,
}

impl<K: Clone + Eq + Hash> Forest<K> {
    fn build(nodes: &[TreeNode<K>]) -> Self {
        let mut index = HashMap::new();
        let (mut keys, mut src) = (Vec::new(), Vec::new());
        // A key used twice belongs to its first node.
        for (at, node) in nodes.iter().enumerate() {
            if !index.contains_key(&node.key) {
                index.insert(node.key.clone(), keys.len());
                keys.push(node.key.clone());
                src.push(at);
            }
        }
        let parent: Vec<Option<usize>> = src
            .iter()
            .map(|at| {
                nodes[*at]
                    .parent
                    .as_ref()
                    .and_then(|parent| index.get(parent).copied())
            })
            .collect();
        let mut children = vec![Vec::new(); keys.len()];
        let mut roots = Vec::new();
        for (node, parent) in parent.iter().enumerate() {
            match parent {
                Some(parent) => children[*parent].push(node),
                None => roots.push(node),
            }
        }
        // A node on a cycle, and a node below one, has no path from a root.
        let mut order = Vec::with_capacity(keys.len());
        let mut stack: Vec<usize> = roots.iter().rev().copied().collect();
        while let Some(node) = stack.pop() {
            order.push(node);
            stack.extend(children[node].iter().rev().copied());
        }
        Self {
            keys,
            src,
            parent,
            children,
            roots,
            order,
            index,
        }
    }
}

/// The shown rows: depth first, the children of a closed node left out. With
/// a `query`, only the nodes whose label contains it and their ancestors,
/// all open.
/// Flatten a forest in depth-first order, retaining the input order of siblings.
pub fn flatten_tree_rows<K: Clone + Eq + Hash>(
    nodes: &[TreeNode<K>],
    expanded: &HashSet<K>,
) -> Vec<TreeRow<K>> {
    flatten(nodes, expanded, "")
        .into_iter()
        .map(|entry| entry.row)
        .collect()
}

fn flatten<K: Clone + Eq + Hash>(
    nodes: &[TreeNode<K>],
    expanded: &HashSet<K>,
    query: &str,
) -> Vec<Entry<K>> {
    let forest = Forest::build(nodes);
    let count = forest.keys.len();
    let filtering = !query.is_empty();
    let mut keep = vec![true; count];
    if filtering {
        let query = query.to_lowercase();
        keep.fill(false);
        for &node in &forest.order {
            keep[node] = nodes[forest.src[node]]
                .label
                .get()
                .to_lowercase()
                .contains(&query);
        }
        // The reverse of a pre-order reaches a node after its descendants.
        for &node in forest.order.iter().rev() {
            if let (true, Some(parent)) = (keep[node], forest.parent[node]) {
                keep[parent] = true;
            }
        }
    }
    let mut kept_children = vec![0u32; count];
    let mut kept_roots = 0u32;
    for &node in &forest.order {
        if keep[node] {
            match forest.parent[node] {
                Some(parent) => kept_children[parent] += 1,
                None => kept_roots += 1,
            }
        }
    }
    let (mut level, mut placed, mut open) =
        (vec![0u32; count], vec![0u32; count], vec![false; count]);
    let mut placed_roots = 0u32;
    let mut out = Vec::new();
    for &node in &forest.order {
        if !keep[node] {
            continue;
        }
        let (parent_level, set_size, pos_in_set, shown) = match forest.parent[node] {
            Some(parent) => {
                placed[parent] += 1;
                (
                    level[parent],
                    kept_children[parent],
                    placed[parent],
                    open[parent],
                )
            }
            None => {
                placed_roots += 1;
                (0, kept_roots, placed_roots, true)
            }
        };
        level[node] = parent_level + 1;
        let branch = if kept_children[node] == 0 {
            TreeBranch::Leaf
        } else if filtering || expanded.contains(&forest.keys[node]) {
            TreeBranch::Expanded
        } else {
            TreeBranch::Collapsed
        };
        // The children of a row are shown when the row is and is open.
        open[node] = shown && branch.is_expanded();
        if shown {
            let source = &nodes[forest.src[node]];
            out.push(Entry {
                row: TreeRow {
                    key: forest.keys[node].clone(),
                    level: level[node],
                    set_size,
                    pos_in_set,
                    branch,
                },
                label: source.label.get(),
                disabled: source.disabled,
            });
        }
    }
    out
}

type TreeMarker<K> = PhantomData<(fn() -> K, kivact_core::custom::UiThreadOnly)>;

struct Shared<K: kivact_core::state::ItemKey> {
    nodes: Value<Vec<TreeNode<K>>>,
    expanded: Binding<Vec<K>>,
    list: ListApi<K>,
    filter: Signal<Reactive<String>>,
    rows: Value<Vec<TreeRow<K>>>,
    info: Value<HashMap<K, Entry<K>>>,
    activate: Cell<Option<Callback<K>>>,
}

/// Actual input, selection and reactive metadata of an attached tree row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeItemState {
    pub input: PartState,
    pub selected: bool,
    pub active: bool,
    pub branch: TreeBranch,
    pub level: u32,
}

/// Unstyled behavior and semantics of a tree view over a flat list of
/// [`TreeNode`]s: the rows to draw, the selection, the expansion and the keys
/// of the ARIA authoring practices tree view.
///
/// The application owns the expanded keys and the selected keys through two
/// bindings, and the data through the nodes. A kit decides how a row looks
/// (its indentation, its marker, its button to open and close) and `TreeApi`
/// adds the role, the state and the behavior. `TreeApi` is `Copy`, so one value
/// serves the container and every row. [`rows`](Self::rows) is the list to
/// draw, in tree order. Key the `for_each` by `(key, branch.has_children())`: a row
/// reads its level, position, expanded state and label as they change, but
/// whether it is a parent is decided when it is built.
///
/// Roving: the tree is one tab stop. The supplied [`Selection`] chooses
/// single or multiple selection. Disabled nodes are shown, but the keys
/// skip them and they cannot be selected.
///
/// Keys on the focused row (they stop at the ends and do not wrap):
///
/// | Keys | Effect |
/// |---|---|
/// | Down, Up | The next or previous shown row, selecting only it |
/// | Home, End, Page Up, Page Down | As in a list |
/// | Right | On a closed parent: open it. On an open parent: go to its first child. On a leaf: nothing |
/// | Left | On an open parent: close it. Otherwise: go to the parent (nothing on a root) |
/// | `*` | Open every sibling of the row (the nodes with the same parent) |
/// | Enter | [`on_activate`](Self::on_activate) with the row's key; it does not open or close |
/// | Shift, Ctrl or Cmd + an arrow, Space, Ctrl or Cmd + A, a character | As in a list: a range in the order of the shown rows, a focus move, a toggle, select all, typeahead on the labels |
///
/// Going to a row selects only it. While a filter is set every parent is
/// shown open, so Left goes to the parent and does not close it.
///
/// A row does not offer the assistive-technology action Activate, because a
/// click would run it too. Assistive technology selects a row (Select), opens
/// or closes a parent (Expand, Collapse), and activates with Enter. A primary
/// press selects the row as in a list, and a double click calls
/// `on_activate` once, on its second press; a single click never does.
///
/// When the active row is hidden by a closing parent (by a key, assistive
/// technology or a change of the data), the nearest ancestor that is still
/// shown becomes the active row, and gets the focus if the focus was on the
/// row that went. Nodes on a cycle, and nodes below one, are not shown and not
/// reported. A key used by several nodes belongs to the first one, and a node
/// whose parent is not among the nodes is a root.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// # pub use kivact_core::{view, state, style};
/// # pub mod prelude {
/// # //! Role: Supply the canonical imports used by this documentation example.
/// # pub use kivact_core::state::{Binding, Callback, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
/// # pub use kivact_core::view::{EditPhase, Component, IntoView, NodeBuilder, Orientation, Role, column, for_each, node, node_ref, row, show, text};
/// # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
/// # }
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{use_tree, TreeNode};
/// use kivact::prelude::*;
///
/// fn files() -> impl IntoView {
///     let nodes = signal(vec![
///         TreeNode::new("src", None, "src"),
///         TreeNode::new("main.rs", Some("src"), "main.rs"),
///         TreeNode::new("README.md", None, "README.md"),
///     ]);
///     let expanded = signal(vec!["src"]);
///     let selected = signal(Vec::<&str>::new());
///     let tree = use_tree(nodes, expanded, Selection::Multiple(selected.into_binding()));
///     let rows = for_each(
///         tree.rows(),
///         |row| (row.key, row.branch.has_children()),
///         move |_, row| {
///             let row = row.get_untracked();
///             let key = row.key;
///             let body = node()
///                 .child(text(key))
///                 .style(style().padding_left(row.level as i32 * 12));
///             body.part(tree.item(key))
///         },
///     );
///     node().part(tree.list()).name("Files").child(rows)
/// }
/// ```
pub struct TreeApi<K: 'static> {
    state: Owned<Rc<dyn std::any::Any>>,
    _m: TreeMarker<K>,
}

impl<K: 'static> Clone for TreeApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: 'static> Copy for TreeApi<K> {}

/// Create a tree with caller-owned expansion and selection bindings.
/// Missing or hidden keys remain in the bindings. Setup only.
#[track_caller]
pub fn use_tree<K: kivact_core::state::ItemKey>(
    nodes: impl IntoValue<Vec<TreeNode<K>>>,
    expanded: impl kivact_core::state::IntoBinding<Vec<K>>,
    selection: impl crate::IntoSelection<K>,
) -> TreeApi<K> {
    let selection = selection.into_selection();
    let owner = Owner::current();
    let nodes = nodes.into_value();
    let expanded = expanded.into_binding();
    let filter = signal(Reactive::<String>::new(String::new()));
    let entries = {
        memo(move || {
            let open: HashSet<K> = expanded.with(|open| open.iter().cloned().collect());
            let query = filter.with(|query| query.get());
            nodes.with(|nodes| flatten(nodes, &open, &query))
        })
    };
    let rows = memo(move || {
        entries.with(|entries| entries.iter().map(|e| e.row.clone()).collect::<Vec<_>>())
    });
    let info = memo(move || {
        entries.with(|entries| {
            entries
                .iter()
                .map(|e| (e.row.key.clone(), e.clone()))
                .collect::<HashMap<_, _>>()
        })
    });
    let keys = memo(move || {
        entries.with(|entries| {
            entries
                .iter()
                .filter(|e| !e.disabled)
                .map(|e| e.row.key.clone())
                .collect::<Vec<_>>()
        })
    });
    let items = memo(move || {
        entries.with(|entries| {
            entries
                .iter()
                .map(|entry| {
                    ListItem::new(entry.row.key.clone(), entry.label.clone())
                        .disabled(entry.disabled)
                })
                .collect::<Vec<_>>()
        })
    });
    let list = use_list(
        items,
        selection,
        ListOptions {
            typeahead: true,
            ..ListOptions::default()
        },
    );
    // The active row that a closing parent hid is replaced by the
    // nearest ancestor that is still shown.
    let list = list.leave_to(move |gone: &K| {
        nodes.with(|nodes| {
            let forest = Forest::build(nodes);
            let mut at = forest.index.get(gone).copied();
            while let Some(node) = at {
                at = forest.parent[node];
                if let Some(parent) = at
                    && keys.with(|keys| keys.contains(&forest.keys[parent]))
                {
                    return Some(forest.keys[parent].clone());
                }
            }
            None
        })
    });
    let shared = Rc::new(Shared {
        nodes,
        expanded,
        list,
        filter,
        rows,
        info,
        activate: Cell::new(None),
    });
    TreeApi {
        state: owner.store("TreeApi", shared as Rc<dyn std::any::Any>),
        _m: PhantomData,
    }
}

impl<K: kivact_core::state::ItemKey> TreeApi<K> {
    /// Drawing state of the mounted tree container.
    pub fn list_state(self) -> Value<PartState> {
        self.state().list.list_state()
    }
    /// Drawing state and branch metadata of a tree item.
    pub fn item_state(self, key: K) -> Value<TreeItemState> {
        let shared = self.state();
        let input = shared.list.item_state(key.clone());
        memo(move || {
            let input = input.get();
            shared.info.with(|info| TreeItemState {
                input: input.input,
                selected: input.selected,
                active: input.active,
                branch: info
                    .get(&key)
                    .map_or(TreeBranch::Leaf, |entry| entry.row.branch),
                level: info.get(&key).map_or(0, |entry| entry.row.level),
            })
        })
    }

    #[track_caller]
    fn state(self) -> Rc<Shared<K>> {
        self.state
            .get()
            .downcast::<Shared<K>>()
            .expect("invariant: controller state has its original type")
    }

    /// Configure modifier gestures for multiple selection. Setup only.
    #[track_caller]
    pub fn policy(self, policy: ListRangeOptions) -> Self {
        self.state().list.retune(|list| list.range = policy);
        self
    }

    /// Show only the nodes whose label contains `query` (ignoring case) and
    /// their ancestors, every one of them open for as long as the query is
    /// not empty. The bindings of the expanded and selected keys are not
    /// written by it. Setup only.
    #[track_caller]
    pub fn filter(self, query: impl IntoValue<String>) -> Self {
        self.state().filter.set(Reactive::<String>::new(query));
        self
    }

    /// Call `f` with the key of a row on Enter and on a double click.
    /// Opening and closing a parent is separate: Left, Right, and the kit's
    /// button, which calls [`toggle`](Self::toggle). Setup only.
    #[track_caller]
    pub fn on_activate<R: HandlerResult>(self, f: impl Fn(K) -> R + 'static) -> Self {
        self.state().activate.set(Some(callback(f)));
        self
    }

    /// The shown rows, in tree order: parents before their children, the
    /// children of a closed parent left out. A kit draws them with
    /// `for_each`, as in the [example](Self).
    #[track_caller]
    pub fn rows(self) -> Value<Vec<TreeRow<K>>> {
        self.state().rows
    }

    /// The visible row's label. Setup only.
    #[track_caller]
    pub fn label(self, key: K) -> Value<String> {
        let info = self.state().info;
        memo(move || {
            info.with(|info| {
                info.get(&key)
                    .map_or_else(String::new, |row| row.label.clone())
            })
        })
    }

    /// For each visible row, whether each ancestor is its last sibling.
    /// The flags run from the root to the immediate parent. Setup only.
    #[track_caller]
    pub fn ancestor_last(self) -> Value<HashMap<K, Vec<bool>>> {
        let rows = self.rows();
        memo(move || {
            rows.with(|rows| {
                let mut ancestors = Vec::new();
                let mut result = HashMap::with_capacity(rows.len());
                for row in rows {
                    ancestors.truncate(row.level.saturating_sub(1) as usize);
                    result.insert(row.key.clone(), ancestors.clone());
                    ancestors.push(row.is_last());
                }
                result
            })
        })
    }

    /// The row that has the focus, or had it last: the only row that is a
    /// tab stop. `None` when no row can take it.
    #[track_caller]
    pub fn active(self) -> Value<Option<K>> {
        self.state().list.active()
    }

    /// Open the parent `key` if it is closed and close it if it is open: the
    /// action of a kit's button on a row. It writes the expanded keys (a
    /// filter keeps showing the parent open).
    #[track_caller]
    pub fn toggle(self, key: K) {
        let state = self.state();
        let open = state.expanded.with_untracked(|open| open.contains(&key));
        Self::set_open(&state, &key, !open);
    }

    #[track_caller]
    fn set_open(state: &Shared<K>, key: &K, open: bool) {
        let mut keys = state.expanded.get_untracked();
        if keys.contains(key) == open {
            return;
        }
        if open {
            keys.push(key.clone());
        } else {
            keys.retain(|k| k != key);
        }
        state.expanded.set(keys);
    }

    /// Selection projected from the caller's single or multiple binding.
    #[track_caller]
    pub fn selected(self) -> Value<Vec<K>> {
        self.state().list.selected()
    }

    /// Attach one visible row's semantics and behavior to a node or region.
    /// Metadata updates reactively. Hidden and absent rows are ineligible.
    /// Key caller `for_each` rows by `(key, branch.has_children())`: parent/leaf changes
    /// must rebuild the Part because Expand and Collapse are set at setup.
    /// A canvas caller likewise replaces declaration identity on that change.
    #[track_caller]
    pub fn item(self, key: K) -> Part<TreeItemState> {
        self.item_part(key, true)
    }

    /// Attach a row that the presenter removes structurally as soon as it
    /// leaves [`Self::rows`]. Allows the host to capture its painted exit
    /// before removal. The presenter must never leave the live row mounted
    /// when it is absent from the logical tree.
    #[track_caller]
    pub fn item_with_exit(self, key: K) -> Part<TreeItemState> {
        self.item_part(key, false)
    }

    fn item_part(self, key: K, hide_absent: bool) -> Part<TreeItemState> {
        let this = self;
        let state = self.state();
        let info = state.info;
        let live = memo({
            let key = key.clone();
            move || info.with(|info| info.get(&key).cloned())
        });
        let parent = live.with_untracked(|entry| {
            entry
                .as_ref()
                .is_some_and(|entry| entry.row.branch.has_children())
        });
        let for_key = key.clone();
        let for_pointer = key.clone();
        let base = state
            .list
            .item(key.clone())
            .map_state(move |state| TreeItemState {
                input: state.input,
                selected: state.selected,
                active: state.active,
                branch: live.with(|entry| {
                    entry
                        .as_ref()
                        .map_or(TreeBranch::Leaf, |entry| entry.row.branch)
                }),
                level: live.with(|entry| entry.as_ref().map_or(0, |entry| entry.row.level)),
            });
        let part_state = base.state();
        Part::new(part_state, move |host| {
            let mut part = host
                .part(base)
                .role(Role::TreeItem)
                .hidden(move || hide_absent && live.with(Option::is_none))
                .level(move || live.with(|entry| entry.as_ref().map_or(0, |entry| entry.row.level)))
                .set_size(move || {
                    live.with(|entry| entry.as_ref().map_or(0, |entry| entry.row.set_size))
                })
                .pos_in_set(move || {
                    live.with(|entry| entry.as_ref().map_or(0, |entry| entry.row.pos_in_set))
                })
                .on_key(move |event: KeyEvent| this.on_key(&for_key, &event))
                .on_pointer(move |event: PointerEvent| {
                    if is_double_press(&event) {
                        this.activate(&for_pointer);
                    }
                    Handled::No
                });
            if parent {
                let for_open = key.clone();
                part = part
                    .expanded(move || {
                        live.with(|entry| {
                            entry
                                .as_ref()
                                .is_some_and(|entry| entry.row.branch.is_expanded())
                        })
                    })
                    .a11y_action(A11yAction::Expand, move |_| {
                        Self::set_open(&this.state(), &for_open, true);
                    })
                    .a11y_action(A11yAction::Collapse, move |_| {
                        Self::set_open(&this.state(), &key, false);
                    });
            }
            part
        })
    }

    /// Attach the tree role and list container semantics. The caller names it.
    #[track_caller]
    pub fn list(self) -> Part<PartState> {
        let base = self.state().list.list();
        let part_state = base.state();
        Part::new(part_state, move |host| host.part(base).role(Role::Tree))
    }

    #[track_caller]
    fn activate(self, key: &K) {
        if let Some(callback) = self.state().activate.get() {
            callback.call(key.clone());
        }
    }

    #[track_caller]
    fn on_key(self, from: &K, event: &KeyEvent) -> Handled {
        let state = self.state();
        let m = event.modifiers();
        // A key from a control inside the row is the control's.
        if m.ctrl || m.alt || m.meta || !state.list.holds_focus(from) {
            return Handled::No;
        }
        match event.key() {
            Key::ArrowRight => Self::right(&state, from),
            Key::ArrowLeft => Self::left(&state, from),
            Key::Char('*') => Self::open_siblings(&state, from),
            Key::Enter if !m.shift && state.activate.get().is_some() => {
                self.activate(from);
                Handled::Yes
            }
            _ => Handled::No,
        }
    }

    fn entry(state: &Shared<K>, key: &K) -> Option<Entry<K>> {
        state.info.with_untracked(|info| info.get(key).cloned())
    }

    /// Go to `key`, unless the keys skip it (disabled).
    #[track_caller]
    fn go(state: &Shared<K>, key: &K) {
        if Self::entry(state, key).is_some_and(|e| !e.disabled) {
            state.list.choose(key.clone());
        }
    }

    #[track_caller]
    fn right(state: &Shared<K>, from: &K) -> Handled {
        let Some(entry) = Self::entry(state, from) else {
            return Handled::No;
        };
        if !entry.row.branch.has_children() {
            return Handled::Yes;
        }
        if !entry.row.branch.is_expanded() {
            Self::set_open(state, from, true);
            return Handled::Yes;
        }
        // The first child that the keys reach.
        let child = state.rows.with_untracked(|rows| {
            let at = rows.iter().position(|row| row.key == *from)?;
            let level = entry.row.level;
            rows[at + 1..]
                .iter()
                .take_while(|row| row.level > level)
                .filter(|row| row.level == level + 1)
                .find(|row| Self::entry(state, &row.key).is_some_and(|e| !e.disabled))
                .map(|row| row.key.clone())
        });
        if let Some(child) = child {
            Self::go(state, &child);
        }
        Handled::Yes
    }

    #[track_caller]
    fn left(state: &Shared<K>, from: &K) -> Handled {
        let Some(entry) = Self::entry(state, from) else {
            return Handled::No;
        };
        let filtering = state
            .filter
            .with_untracked(|query| !query.get_untracked().is_empty());
        if entry.row.branch.is_expanded() && !filtering {
            Self::set_open(state, from, false);
            return Handled::Yes;
        }
        let parent = state.nodes.with_untracked(|nodes| {
            let forest = Forest::build(nodes);
            let node = *forest.index.get(from)?;
            forest.parent[node].map(|parent| forest.keys[parent].clone())
        });
        if let Some(parent) = parent {
            Self::go(state, &parent);
        }
        Handled::Yes
    }

    fn open_siblings(state: &Shared<K>, from: &K) -> Handled {
        let parents = state.nodes.with_untracked(|nodes| {
            let forest = Forest::build(nodes);
            let node = *forest.index.get(from)?;
            let siblings = match forest.parent[node] {
                Some(parent) => &forest.children[parent],
                None => &forest.roots,
            };
            Some(
                siblings
                    .iter()
                    .filter(|sibling| !forest.children[**sibling].is_empty())
                    .map(|sibling| forest.keys[*sibling].clone())
                    .collect::<Vec<_>>(),
            )
        });
        let Some(parents) = parents else {
            return Handled::No;
        };
        let mut keys = state.expanded.get_untracked();
        let before = keys.len();
        for parent in parents {
            if !keys.contains(&parent) {
                keys.push(parent);
            }
        }
        if keys.len() != before {
            state.expanded.set(keys);
        }
        Handled::Yes
    }
}
