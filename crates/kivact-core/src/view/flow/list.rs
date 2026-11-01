//! Role: Keyed and indexed lists retain item identity across view updates.
//! Rows keep their
//! owner while their key (or index) stays; changed items only update the
//! row's `Value<T>`; order changes use the fewest moves (LIS).
//!
//! The row builders receive only the key (or index) and the row's
//! `Value<T>`.
//!
//! Invariants:
//! - Items and keys are read in a pure section; rows are mounted, moved
//!   and disposed outside it.
//! - Each row is set up in a `Setup` frame of its own owner (untracked); a
//!   panic there reports to the row owner and mounts an empty fragment.
//! - Creates are emitted in id order, inserts right to left (hosts rely on
//!   increasing ids).
//! - `for_each` updates rows by `PartialEq` alone while the keys keep their
//!   order; when the structure changes it matches keys with a `HashMap`,
//!   keeps the rows of a longest increasing subsequence in place and
//!   `Move`s the others right to left. New rows are made in the engine and
//!   inserted right to left directly. Editing one item scans the keys in
//!   O(N).

use std::cell::RefCell;
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

use kivact_wire::NodeId;

use crate::view::boundary::mount_in;
use crate::view::descriptor::{IntoView, View, view_fn};
use crate::view::flow::branch::{Branch, mount_branch, unmount_branch};
use crate::view::state::ViewState;
use kivact_reactive::keyed::{self, Diff, Plan, Rows};
use kivact_reactive::{EffectKind, ErrorKind, IntoValue, Owner, Runtime, Signal, UiError, Value};

/// A row builder of a keyed list.
pub(crate) type RowFn<K, T> = Rc<dyn Fn(K, Value<T>) -> View>;

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

pub(crate) use kivact_reactive::keyed::key_hash;

/// Indices (into `seq`) of one longest strictly increasing subsequence.
pub(crate) fn lis(seq: &[usize]) -> Vec<usize> {
    let mut tails: Vec<usize> = Vec::new();
    let mut prev = vec![usize::MAX; seq.len()];
    for (i, &v) in seq.iter().enumerate() {
        let pos = tails.partition_point(|&t| seq[t] < v);
        if pos > 0 {
            prev[i] = tails[pos - 1];
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
        cur = (prev[i] != usize::MAX).then(|| prev[i]);
    }
    out.reverse();
    out
}

/// Keyed list: `row(key, item)` for each item. Rows keep their owner (and
/// the state their components create) while their key stays; changed items
/// update the row's `Value<T>`; order changes use the fewest moves (LIS).
/// An update scans the keys once (O(N)) and clones only changed or new items.
///
/// ```ignore
/// fn task_row(id: u64, task: Value<Task>) -> impl IntoView { .. }
/// for_each(tasks, |t| t.id, task_row)
/// ```
#[track_caller]
pub fn for_each<T, K, V>(
    items: impl IntoValue<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    row: impl Fn(K, Value<T>) -> V + 'static,
) -> View
where
    T: Clone + PartialEq + 'static,
    K: Eq + Hash + Clone + 'static,
    V: IntoView,
{
    let at = Location::caller();
    let items = items.into_prop();
    let row: RowFn<K, T> = Rc::new(move |k, v| row(k, v).into_view());
    view_fn(move |cx| {
        let frag = cx.fragment();
        let weak = cx.weak();
        let parent = cx.owner;
        let state: RefCell<Rows<K, Row<K, T>>> = RefCell::new(Rows::new());
        cx.effect(EffectKind::Internal, at, move || {
            let Some(rt) = weak.upgrade() else { return };
            // Items and keys only compute values: pure.
            let diff = kivact_reactive::pure(|| {
                items.with(|list| keyed::diff(&state.borrow(), list, &key))
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
                    apply_list(&rt, parent, frag, &state, keys, plans, &*row, at);
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
fn apply_list<T: Clone + 'static, K: Eq + Hash + Clone + 'static>(
    rt: &Rc<Runtime>,
    parent: Owner,
    frag: NodeId,
    state: &RefCell<Rows<K, Row<K, T>>>,
    keys: Vec<K>,
    plans: Vec<Plan<T>>,
    row: &dyn Fn(K, Value<T>) -> View,
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
    for (i, slot) in old.iter_mut().enumerate() {
        if !keep_mark[i]
            && let Some(r) = slot.take()
        {
            let removed = ViewState::tree_mut(rt).remove_unlinked(r.node);
            drop(removed);
            r.row.owner().dispose();
        }
    }
    // 3. Stable rows = LIS of old positions in new order.
    let mut stable = vec![false; old.len()];
    if kept_old_pos.windows(2).all(|w| w[0] < w[1]) {
        for &o in &kept_old_pos {
            stable[o] = true;
        }
    } else {
        for i in lis(&kept_old_pos) {
            stable[kept_old_pos[i]] = true;
        }
    }
    // 4. Build the new row list; mount new rows; attach and move right to left.
    let live = ViewState::tree(rt).is_live(frag);
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
                let node = mount_in(rt, keyed.owner(), || row(k, item.into_value()), at);
                if let Some(nd) = ViewState::tree_mut(rt).get_mut(node) {
                    nd.parent = Some(frag);
                }
                new_rows.push(Row { row: keyed, node });
                fresh.push(true);
                moved.push(false);
            }
        }
    }
    let children: Vec<NodeId> = new_rows.iter().map(|r| r.node).collect();
    ViewState::tree_mut(rt).set_children(frag, children);
    if live && (fresh.iter().any(|f| *f) || moved.iter().any(|m| *m)) {
        let mut tree = ViewState::tree_mut(rt);
        // Place the already-created rows from right to left.
        let mut next = tree.next_host_after(frag);
        for i in (0..new_rows.len()).rev() {
            let r = &new_rows[i];
            if fresh[i] {
                tree.attach_writes_before(r.node, next);
            } else if moved[i] {
                tree.move_writes_before(r.node, next);
            }
            if let Some(h) = tree.first_host_of(r.node) {
                next = Some(h);
            }
        }
    }
    state.borrow_mut().replace(new_rows);
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
    T: Clone + PartialEq + 'static,
    V: IntoView,
{
    let at = Location::caller();
    let items = items.into_prop();
    view_fn(move |cx| {
        let frag = cx.fragment();
        let weak = cx.weak();
        let parent = cx.owner;
        let rows: RefCell<Vec<(Branch, Signal<T>)>> = RefCell::new(Vec::new());
        cx.effect(EffectKind::Internal, at, move || {
            let Some(rt) = weak.upgrade() else { return };
            let (updates, appended, len) = kivact_reactive::pure(|| {
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
                let cell = RefCell::new(None);
                let init = RefCell::new(Some(item));
                let b = mount_branch(
                    &rt,
                    parent,
                    frag,
                    None,
                    &|| {
                        let Some(v) = init.borrow_mut().take() else {
                            return View::empty();
                        };
                        let sig = kivact_reactive::signal(v);
                        *cell.borrow_mut() = Some(sig);
                        row(i, sig.into_value()).into_view()
                    },
                    at,
                );
                if let Some(sig) = cell.into_inner() {
                    rows.borrow_mut().push((b, sig));
                }
            }
        });
        frag
    })
}
