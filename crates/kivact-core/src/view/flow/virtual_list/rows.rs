//! Role: Row wrappers: mounting, reordering and removing the visible rows.

use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;
use kivact_runtime::driver::Runtime;
use kivact_runtime::{Owner, Signal, Trigger};
use kivact_types::{Edges, LayoutProps, Length, NodeId, Position};

use super::state::{VRow, VState};
use crate::view::boundary::mount_in;
use crate::view::descriptor::Mcx;
use crate::view::flow::list::{RowFn, lis};
use crate::view::state::ViewState;

pub(super) fn px(v: f64) -> Length {
    Length::Px(v)
}

fn wrapper_layout(top: f64, height: Option<f64>) -> LayoutProps {
    LayoutProps {
        position: Position::Absolute,
        inset: Edges {
            top: px(top),
            left: px(0.0),
            right: px(0.0),
            bottom: Length::Auto,
        },
        height: height.map(px).unwrap_or(Length::Auto),
        ..LayoutProps::default()
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "internal mount helper; the arguments are the list's parts"
)]
pub(super) fn apply_visible<K: kivact_runtime::ItemKey, T: Clone + 'static>(
    rt: &Rc<Runtime>,
    parent: Owner,
    window: kivact_types::WindowId,
    spacer: NodeId,
    state: &Rc<RefCell<VState<K, T>>>,
    visible: Vec<(K, Option<T>, f64, Option<f64>)>,
    row: &RowFn<K, T>,
    measured_version: Trigger,
    at: &'static Location<'static>,
) {
    remove_hidden_rows(rt, state, &visible);
    // Wrappers in item order, and which of them are new.
    let mut order: Vec<NodeId> = Vec::with_capacity(visible.len());
    let mut fresh: Vec<bool> = Vec::with_capacity(visible.len());
    for (k, item, top, h) in visible {
        let existing = state
            .borrow()
            .rows
            .get(&k)
            .map(|r| (r.sig, r.wrapper, r.top));
        match existing {
            Some((sig, wrapper, old_top)) => {
                order.push(wrapper);
                fresh.push(false);
                if let Some(v) = item {
                    sig.update(move |slot| *slot = v);
                }
                if old_top != top {
                    ViewState::tree_mut(rt).update(
                        wrapper,
                        PropValue::Layout(wrapper_layout(top, h)),
                        rt.running_effect(),
                    );
                    if let Some(r) = state.borrow_mut().rows.get_mut(&k) {
                        r.top = top;
                    }
                }
            }
            None => {
                let Some(item) = item else { continue };
                let owner = parent.child(at);
                let sig: Signal<T> = owner.signal(item, at);
                let cx = Mcx { rt, owner, window };
                let wrapper = cx.host(NodeType::Box);
                cx.update(wrapper, PropValue::Layout(wrapper_layout(top, h)));
                if h.is_none() {
                    let st = Rc::downgrade(state);
                    let key = k.clone();
                    let mut tree = ViewState::tree_mut(rt);
                    tree.set_observe(wrapper, true);
                    if let Some(hd) = tree.handlers_mut(wrapper) {
                        hd.observe_layout(move |_, size| {
                            let Some(st) = st.upgrade() else { return };
                            let mut s = st.borrow_mut();
                            let prev = s.measured.insert(key.clone(), size.height);
                            if prev != Some(size.height) {
                                if let Some(&i) = s.index_of.get(&key) {
                                    s.heights.set(i, size.height.max(1.0));
                                }
                                drop(s);
                                measured_version.notify();
                            }
                        });
                    }
                }
                let content = mount_in(rt, owner, window, || row.call((k.clone(), sig)));
                cx.append(wrapper, content);
                order.push(wrapper);
                fresh.push(true);
                state.borrow_mut().rows.insert(
                    k,
                    VRow {
                        owner,
                        wrapper,
                        sig,
                        top,
                    },
                );
            }
        }
    }
    reorder_rows(rt, spacer, order, &fresh);
}

/// Make the host children of `spacer` follow the item order, so reading and
/// Tab order match the visual order. Kept rows move as little as
/// possible (LIS of their old positions); new rows are inserted in place.
fn reorder_rows(rt: &Runtime, spacer: NodeId, order: Vec<NodeId>, fresh: &[bool]) {
    let tree = ViewState::tree(rt);
    let old = tree.reader().children(spacer);
    if old == order {
        return;
    }
    let pos: HashMap<NodeId, usize> = old.iter().enumerate().map(|(i, n)| (*n, i)).collect();
    let kept: Vec<(usize, usize)> = order
        .iter()
        .enumerate()
        .filter(|(i, _)| !fresh[*i])
        .filter_map(|(i, n)| pos.get(n).map(|p| (i, *p)))
        .collect();
    let seq: Vec<usize> = kept.iter().map(|(_, p)| *p).collect();
    let mut stable = vec![false; order.len()];
    if seq.windows(2).all(|w| w[0] < w[1]) {
        for (i, _) in &kept {
            stable[*i] = true;
        }
    } else {
        for j in lis(&seq) {
            stable[kept[j].0] = true;
        }
    }
    let live = tree.contains(spacer);
    if !live {
        return;
    }
    let mut next: Option<NodeId> = None;
    for i in (0..order.len()).rev() {
        let w = order[i];
        if fresh[i] {
            tree.engine().insert(spacer, w, next);
        } else if !stable[i] {
            tree.engine().move_before(w, next);
        }
        next = Some(w);
    }
}

fn remove_hidden_rows<K: kivact_runtime::ItemKey, T: 'static>(
    rt: &Runtime,
    state: &RefCell<VState<K, T>>,
    visible: &[(K, Option<T>, f64, Option<f64>)],
) {
    let keep: std::collections::HashSet<K> = visible.iter().map(|(k, ..)| k.clone()).collect();
    let gone: Vec<VRow<T>> = {
        let mut s = state.borrow_mut();
        let dead: Vec<K> = s
            .rows
            .keys()
            .filter(|k| !keep.contains(*k))
            .cloned()
            .collect();
        dead.into_iter().filter_map(|k| s.rows.remove(&k)).collect()
    };
    for r in gone {
        let removed = ViewState::tree_mut(rt).remove(r.wrapper);
        drop(removed);
        r.owner.dispose();
    }
}
