//! Role: Keyed and indexed lists retain item identity across view updates.
//! Rows keep their
//! owner while their key (or index) stays; changed items only update the
//! row's `Value<T>`; order changes use the fewest moves (LIS).
//!
//! A `for_each` or `virtual_list` row builder receives one row handle
//! (`ListSource::Row`): a `Value<T>`, or a `Binding<T>` for a writable
//! source. A `for_index` row builder receives the index and the row's `Value<T>`.
//!
//! Invariants:
//! - Items and keys are read in a pure section; rows are mounted, moved
//!   and disposed outside it.
//! - Each row is set up in a `Setup` frame of its own owner (untracked); a
//!   panic there propagates.
//! - Creates are emitted in id order, inserts right to left (hosts rely on
//!   increasing ids).
//! - `for_each` updates rows by `PartialEq` alone while the keys keep their
//!   order; when the structure changes it matches keys with a `HashMap`,
//!   keeps the rows of a longest increasing subsequence in place and
//!   `Move`s the others right to left. New rows are made in the engine and
//!   inserted right to left directly. Editing one item through a writable
//!   row scans the keys in O(N).

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_types::NodeId;

use crate::view::boundary::mount_in;
use crate::view::descriptor::{IntoView, View, view_fn};
use crate::view::flow::branch::{Branch, mount_branch, unmount_branch};
use crate::view::state::ViewState;
use kivact_runtime::keyed::{self, Diff, Plan, Rows};
use kivact_runtime::{
    Binding, EffectKind, ErrorKind, IntoBinding, IntoValue, ItemKey, Owner, Prop, PropValue,
    Runtime, Signal, UiError, Value,
};

/// A row builder of a keyed list: the key and the row's item signal.
pub(crate) type RowFn<K, T> = crate::view::ViewFn<(K, Signal<T>)>;

/// The per-row conversion of a normalized list source.
type MakeRow<T, R> = Rc<dyn Fn(Signal<T>, Rc<dyn Fn(&T, &T) -> bool>) -> R>;

/// Wraps a user row function: converts the row's item signal into the
/// source's row handle (`ListSource::Row`) with the key comparison the
/// handle needs to find its slot, then calls `row` with it. Shared by
/// `for_each` and `virtual_list`, so both keyed lists give rows one shape.
pub(crate) fn source_row<T, K, V, R>(
    key: Rc<impl Fn(&T) -> K + 'static + ?Sized>,
    make_row: MakeRow<T, R>,
    row: impl Fn(R) -> V + 'static,
) -> RowFn<K, T>
where
    T: 'static,
    K: PartialEq + 'static,
    V: IntoView,
    R: 'static,
{
    let same_key: Rc<dyn Fn(&T, &T) -> bool> = Rc::new(move |a, b| key(a) == key(b));
    crate::view::ViewFn::from(move |(_k, value): (K, Signal<T>)| {
        row(make_row(value, same_key.clone()))
    })
}

/// A mounted row: the keyed row (owner and item) and its node.
struct Row<K, T: 'static> {
    row: keyed::Row<K, T>,
    node: NodeId,
}

impl<K, T: 'static> AsRef<keyed::Row<K, T>> for Row<K, T> {
    fn as_ref(&self) -> &keyed::Row<K, T> {
        &self.row
    }
}

pub(crate) use kivact_runtime::keyed::key_hash;

/// Indices (into `seq`) of one longest strictly increasing subsequence.
pub(crate) fn lis(seq: &[usize]) -> Vec<usize> {
    let mut tails: Vec<usize> = Vec::new();
    let mut prev: Vec<Option<usize>> = vec![None; seq.len()];
    for (i, &v) in seq.iter().enumerate() {
        let pos = tails.partition_point(|&t| seq[t] < v);
        if pos > 0 {
            prev[i] = Some(tails[pos - 1]);
        }
        if pos == tails.len() {
            tails.push(i);
        } else {
            tails[pos] = i;
        }
    }
    let mut out = Vec::with_capacity(tails.len());
    let mut cur = tails.last().copied();
    while let Some(i) = cur {
        out.push(i);
        cur = prev[i];
    }
    out.reverse();
    out
}

/// Keyed list: `row(item)` for each item, where `item` is the source's row
/// handle: a `Value<T>`, or a `Binding<T>` that writes back into a
/// `Signal`/`Binding` source. Rows keep their owner (and the state their
/// components create) while their key stays; changed items update the
/// row's handle; order changes use the fewest moves (LIS). An update scans
/// the keys once (O(N)) and clones only changed or new items.
///
/// ```ignore
/// fn task_row(task: Binding<Task>) -> impl IntoView { .. }
/// for_each(tasks, |t| t.id, task_row)
/// ```
#[track_caller]
pub fn for_each<T, K, V, S>(
    items: S,
    key: impl Fn(&T) -> K + 'static,
    row: impl Fn(S::Row) -> V + 'static,
) -> View
where
    T: PropValue,
    K: ItemKey,
    V: IntoView,
    S: ListSource<T>,
{
    let at = Location::caller();
    let ListInput {
        items,
        row: make_row,
    } = items.into_list_source();
    let key = Rc::new(key);
    let row = source_row(key.clone(), make_row, row);
    view_fn(move |cx| {
        let frag = cx.fragment();
        let weak = cx.weak();
        let window = cx.window;
        let parent = cx.owner;
        let state: RefCell<Rows<K, Row<K, T>>> = RefCell::new(Rows::new());
        cx.effect(EffectKind::Internal, at, move || {
            let Some(rt) = weak.upgrade() else { return };
            // Items and keys only compute values: pure.
            let diff = kivact_runtime::pure(|| {
                items.with(|list| keyed::diff(&state.borrow(), list, &*key))
            });
            match diff {
                Diff::Values(changed) => {
                    let sigs: Vec<(Signal<T>, T)> = {
                        let st = state.borrow();
                        changed
                            .into_iter()
                            .map(|(i, v)| (st.rows()[i].row.item(), v))
                            .collect()
                    };
                    for (sig, v) in sigs {
                        sig.update(move |slot| *slot = v);
                    }
                }
                Diff::Structure { keys, plans, dups } => {
                    for h in dups {
                        rt.report(
                            None,
                            UiError::new(
                                ErrorKind::DuplicateKey { key_hash: h },
                                format!("duplicate for_each key (hash {h:#x}); later item dropped"),
                                Some(at),
                            ),
                        );
                    }
                    apply_list(&rt, parent, window, frag, &state, keys, plans, &row, at);
                }
            }
        });
        frag
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "internal mount helper; the arguments are the flow's parts"
)]
fn apply_list<T: Clone + 'static, K: kivact_runtime::ItemKey>(
    rt: &Rc<Runtime>,
    parent: Owner,
    window: kivact_types::WindowId,
    frag: NodeId,
    state: &RefCell<Rows<K, Row<K, T>>>,
    keys: Vec<K>,
    plans: Vec<Plan<T>>,
    row: &RowFn<K, T>,
    at: &'static Location<'static>,
) {
    let mut old: Vec<Option<Row<K, T>>> = state.borrow_mut().take().into_iter().map(Some).collect();
    // 1. Update changed rows (values only), collect kept old positions.
    let mut kept_old_pos = Vec::with_capacity(plans.len());
    let mut plans = plans;
    for p in plans.iter_mut() {
        if let Plan::Keep { old: o, changed } = p {
            kept_old_pos.push(*o);
            if let (Some(v), Some(r)) = (changed.take(), old[*o].as_ref()) {
                r.row.item().update(move |slot| *slot = v);
            }
        }
    }
    // 2. Dispose rows whose key disappeared.
    let mut keep_mark = vec![false; old.len()];
    for &o in &kept_old_pos {
        keep_mark[o] = true;
    }
    let gone: Vec<Row<K, T>> = old
        .iter_mut()
        .zip(&keep_mark)
        .filter(|(_, kept)| !**kept)
        .filter_map(|(slot, _)| slot.take())
        .collect();
    let parts: Vec<(NodeId, Owner)> = gone.iter().map(|r| (r.node, r.row.owner())).collect();
    dispose_rows(rt, &parts);
    drop(gone);
    // 3. Stable rows = LIS of old positions in new order.
    let stable = stable_positions(&kept_old_pos, old.len());
    // 4. Build the new row list; mount new rows; attach and move right to left.
    let live = ViewState::tree(rt).contains(frag);
    let n = plans.len();
    let mut new_rows: Vec<Row<K, T>> = Vec::with_capacity(n);
    let mut fresh = Vec::with_capacity(n);
    let mut moved = Vec::with_capacity(n);
    for (p, k) in plans.into_iter().zip(keys) {
        match p {
            Plan::Keep { old: o, .. } => {
                if let Some(r) = old[o].take() {
                    new_rows.push(r);
                    fresh.push(false);
                    moved.push(!stable[o]);
                }
            }
            Plan::New(item) => {
                let Some(keyed) = keyed::Row::new(parent, k.clone(), item, at) else {
                    continue;
                };
                let item = keyed.item();
                // The row's own node: its view's root, or a fragment for an empty view
                // or a group (`View::mount`).
                let node = mount_in(rt, keyed.owner(), window, || row.call((k, item)));
                new_rows.push(Row { row: keyed, node });
                fresh.push(true);
                moved.push(false);
            }
        }
    }
    if live {
        let nodes: Vec<NodeId> = new_rows.iter().map(|r| r.node).collect();
        place_rows(rt, frag, &nodes, &fresh, &moved);
    }
    state.borrow_mut().replace(new_rows);
}

/// Remove each row's node from the tree and dispose its owner, in order
/// (the part of a list update that does not depend on the item type).
#[inline(never)]
fn dispose_rows(rt: &Rc<Runtime>, rows: &[(NodeId, Owner)]) {
    for &(node, owner) in rows {
        let removed = ViewState::tree_mut(rt).remove(node);
        removed.commit(rt);
        owner.dispose();
    }
}

/// Insert the `fresh` rows into `frag` and move the `moved` ones, right to
/// left (`nodes` in the new order; the rows are already created).
#[inline(never)]
fn place_rows(rt: &Rc<Runtime>, frag: NodeId, nodes: &[NodeId], fresh: &[bool], moved: &[bool]) {
    if !fresh.iter().any(|f| *f) && !moved.iter().any(|m| *m) {
        return;
    }
    let tree = ViewState::tree(rt);
    let mut next = None;
    for i in (0..nodes.len()).rev() {
        if fresh[i] {
            tree.engine().insert(frag, nodes[i], next);
        } else if moved[i] {
            tree.engine().move_before(nodes[i], next);
        }
        next = Some(nodes[i]);
    }
}

/// Which old rows stay put: the longest increasing subsequence of their old
/// positions in the new order (every row when the order did not change).
fn stable_positions(kept_old_pos: &[usize], old_len: usize) -> Vec<bool> {
    let mut stable = vec![false; old_len];
    if kept_old_pos.windows(2).all(|w| w[0] < w[1]) {
        for &o in kept_old_pos {
            stable[o] = true;
        }
    } else {
        for i in lis(kept_old_pos) {
            stable[kept_old_pos[i]] = true;
        }
    }
    stable
}

/// Index-keyed list: `row(i, item)` for each item. Row `i` keeps its owner
/// while the list has an item at `i`; the row's `Value<T>` follows the item
/// at that index.
#[track_caller]
pub fn for_index<T, V>(
    items: impl IntoValue<Vec<T>>,
    row: impl Fn(usize, Value<T>) -> V + 'static,
) -> View
where
    T: kivact_runtime::PropValue,
    V: IntoView,
{
    let at = Location::caller();
    let items = items.into_prop();
    view_fn(move |cx| {
        let frag = cx.fragment();
        let weak = cx.weak();
        let window = cx.window;
        let parent = cx.owner;
        let rows: RefCell<Vec<(Branch, Signal<T>)>> = RefCell::new(Vec::new());
        cx.effect(EffectKind::Internal, at, move || {
            let Some(rt) = weak.upgrade() else { return };
            let (updates, appended, len) = kivact_runtime::pure(|| {
                items.with(|list| {
                    let rows = rows.borrow();
                    let mut updates = Vec::new();
                    for (i, (_, sig)) in rows.iter().enumerate().take(list.len()) {
                        if sig.with_untracked(|cur| *cur != list[i]) {
                            updates.push((*sig, list[i].clone()));
                        }
                    }
                    let appended: Vec<T> = list.iter().skip(rows.len()).cloned().collect();
                    (updates, appended, list.len())
                })
            });
            for (sig, v) in updates {
                sig.update(move |slot| *slot = v);
            }
            let removed: Vec<(Branch, Signal<T>)> = {
                let mut r = rows.borrow_mut();
                if r.len() > len {
                    r.split_off(len)
                } else {
                    Vec::new()
                }
            };
            for (b, _) in removed {
                unmount_branch(&rt, b);
            }
            for item in appended {
                let i = rows.borrow().len();
                let owner = parent.child(at);
                let sig = owner.signal(item, at);
                let b = mount_branch(&rt, owner, window, frag, || {
                    row(i, sig.into_value()).into_view()
                });
                rows.borrow_mut().push((b, sig));
            }
        });
        frag
    })
}

/// A list source preserves writability: pass a Signal/Binding for editable rows.
#[diagnostic::on_unimplemented(
    message = "a list source needs a Vec, Signal, Binding, Value, or closure returning Vec",
    note = "pass a Signal or Binding to receive writable Binding rows"
)]
pub trait ListSource<T: PropValue>: Sized + 'static {
    type Row: 'static;
    #[doc(hidden)]
    fn into_list_source(self) -> ListInput<T, Self::Row>;
}
/// Normalized list storage and its per-row conversion.
#[doc(hidden)]
pub struct ListInput<T: 'static, R> {
    pub(crate) items: Prop<Vec<T>>,
    pub(crate) row: MakeRow<T, R>,
}
fn writable_list<T: PropValue>(source: Binding<Vec<T>>) -> ListInput<T, Binding<T>> {
    ListInput {
        items: source.into_prop(),
        row: Rc::new(move |item, same_key| {
            let initial = item.get_untracked();
            // A row reads its own item, which the list keeps current; a
            // write through the row also sets it, so the row reads it back
            // at once.
            kivact_runtime::binding(item, move |replacement: T| {
                let mut list = source.get_untracked();
                if let Some(slot) = list
                    .iter_mut()
                    .find(|candidate| same_key(candidate, &initial))
                    && *slot != replacement
                {
                    *slot = replacement.clone();
                    source.set(list);
                    item.set(replacement);
                }
            })
        }),
    }
}
impl<T: PropValue> ListSource<T> for Signal<Vec<T>> {
    type Row = Binding<T>;
    fn into_list_source(self) -> ListInput<T, Self::Row> {
        writable_list(self.into_binding())
    }
}
impl<T: PropValue> ListSource<T> for Binding<Vec<T>> {
    type Row = Binding<T>;
    fn into_list_source(self) -> ListInput<T, Self::Row> {
        writable_list(self)
    }
}
fn readonly_list<T: PropValue>(items: Prop<Vec<T>>) -> ListInput<T, Value<T>> {
    ListInput {
        items,
        row: Rc::new(|item, _| item.into_value()),
    }
}
impl<T: PropValue> ListSource<T> for Value<Vec<T>> {
    type Row = Value<T>;
    fn into_list_source(self) -> ListInput<T, Self::Row> {
        readonly_list(self.into_prop())
    }
}
impl<T: PropValue> ListSource<T> for Vec<T> {
    type Row = Value<T>;
    fn into_list_source(self) -> ListInput<T, Self::Row> {
        readonly_list(self.into_prop())
    }
}
impl<T: PropValue, F: Fn() -> Vec<T> + 'static> ListSource<T> for F {
    type Row = Value<T>;
    fn into_list_source(self) -> ListInput<T, Self::Row> {
        readonly_list(self.into_prop())
    }
}
