//! Role: The scroll_to request and the visible range of the rows.

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use kivact_runtime::driver::Runtime;
use kivact_runtime::{Prop as InputProp, Signal};
use kivact_types::{NodeId, Point};

use super::builder::RowHeight;
use super::heights::Heights;
use super::state::{Issued, Seek, VState, Viewport};
use crate::view::flow::list::key_hash;
use crate::view::state::ViewState;

/// The scroll offset that brings the row `[top, top + height)` into the
/// viewport `[offset, offset + view)` with the smallest move; a row taller
/// than the viewport aligns to its top (as the host's scroll into view).
fn reveal_offset(top: f64, height: f64, offset: f64, view: f64) -> f64 {
    if top < offset {
        top
    } else {
        offset + (top + height - offset - view).clamp(0.0, top - offset)
    }
}

// xtask-allow(state): legacy: state, viewport
/// What carries out `scroll_to`: the pending request becomes a scroll of the
/// list to the position of the row and, once the row is mounted, a scroll of
/// the host that brings the row into view.
pub(super) struct Seeker<K: 'static, T: 'static> {
    pub(super) items: InputProp<Vec<T>>,
    pub(super) key: Rc<dyn Fn(&T) -> K>,
    pub(super) height: RowHeight,
    pub(super) state: Rc<RefCell<VState<K, T>>>,
    pub(super) viewport: Signal<Option<Viewport>>,
    pub(super) outer: NodeId,
}

/// Scrolls one request may take before it gives up (a `Measured` list whose
/// rows keep changing height under the target).
const MAX_STEPS: u8 = 8;

impl<K: kivact_runtime::ItemKey, T: Clone + 'static> Seeker<K, T> {
    /// Advance the pending request, if any.
    pub(super) fn run(&self, rt: &Rc<Runtime>) {
        let (want, issued, steps) = match &self.state.borrow().seek {
            Seek::Idle => return,
            Seek::Pending { key, issued, steps } => (key.clone(), *issued, *steps),
        };
        if !ViewState::tree(rt).contains(self.outer) {
            return;
        }
        // The row is mounted: the host can bring it into view exactly.
        let mounted = self.state.borrow().rows.get(&want).map(|r| r.wrapper);
        if let Some(wrapper) = mounted.filter(|w| ViewState::tree(rt).contains(*w)) {
            self.finish();
            ViewState::tree(rt).engine().scroll_into_view(wrapper);
            rt.request_turn();
            return;
        }
        // The host has not reported a viewport yet.
        let Some(Viewport {
            offset,
            height: view,
        }) = self.viewport.get_untracked()
        else {
            return;
        };
        if let Some(Issued { from, to }) = issued {
            // The host has not reported the scroll yet.
            if offset == from && offset != to {
                return;
            }
            // The list is somewhere else than the request put it.
            if offset != to {
                self.finish();
                return;
            }
        }
        let place = kivact_runtime::pure(|| {
            self.items.with(|list| {
                let s = self.state.borrow();
                let i = *s.index_of.get(&want)?;
                match self.height {
                    RowHeight::Fixed(h) => {
                        let h = h.max(1.0);
                        Some((i as f64 * h, h, list.len() as f64 * h))
                    }
                    RowHeight::Measured { estimate } => {
                        let top = s.heights.prefix(i);
                        let height = s.measured.get(&want).copied().unwrap_or(estimate).max(1.0);
                        Some((top, height, s.heights.total()))
                    }
                }
            })
        });
        let Some((top, height, total)) = place else {
            self.finish();
            return;
        };
        let target = reveal_offset(top, height, offset, view).min((total - view).max(0.0));
        if target == offset || steps >= MAX_STEPS {
            self.finish();
            return;
        }
        self.state.borrow_mut().seek = Seek::Pending {
            key: want,
            issued: Some(Issued {
                from: offset,
                to: target,
            }),
            steps: steps + 1,
        };
        ViewState::tree(rt)
            .engine()
            .scroll(self.outer, Point { x: 0.0, y: target }, false, None);
        rt.request_turn();
    }

    fn finish(&self) {
        self.state.borrow_mut().seek = Seek::Idle;
    }
}

impl<K: kivact_runtime::ItemKey, T: kivact_runtime::PropValue> Seeker<K, T> {
    /// Rebuild the key index (and the heights of a `Measured` list) from the
    /// current items (tracked). Returns the hashes of duplicate keys.
    pub(super) fn reindex(&self) -> Vec<u64> {
        let estimate = match self.height {
            RowHeight::Measured { estimate } => Some(estimate),
            RowHeight::Fixed(_) => None,
        };
        kivact_runtime::pure(|| {
            self.items.with(|list| {
                let mut s = self.state.borrow_mut();
                let mut dups = Vec::new();
                let mut index_of = HashMap::with_capacity(list.len());
                for (i, it) in list.iter().enumerate() {
                    match index_of.entry((self.key)(it)) {
                        Entry::Occupied(e) => dups.push(key_hash(e.key())),
                        Entry::Vacant(e) => {
                            e.insert(i);
                        }
                    }
                }
                if let Some(estimate) = estimate {
                    let rows = list
                        .iter()
                        .map(|it| {
                            s.measured
                                .get(&(self.key)(it))
                                .copied()
                                .unwrap_or(estimate)
                                .max(1.0)
                        })
                        .collect();
                    s.heights = Heights::build(rows);
                    s.measured.retain(|k, _| index_of.contains_key(k));
                }
                s.index_of = index_of;
                s.indexed = list.len();
                dups
            })
        })
    }

    pub(super) fn visible(
        &self,
        lo: f64,
        hi: f64,
        focused_key: Option<K>,
    ) -> (f64, Vec<(K, Option<T>, f64, Option<f64>)>) {
        let mut visible = Vec::new();
        let total = kivact_runtime::pure(|| {
            self.items.with(|list| {
                let s = self.state.borrow();
                match self.height {
                    RowHeight::Fixed(h) => self.fixed_visible(
                        list,
                        &s,
                        h,
                        (lo, hi),
                        focused_key.as_ref(),
                        &mut visible,
                    ),
                    RowHeight::Measured { .. } => {
                        let len = list.len().min(s.indexed);
                        let first = s.heights.index_at(lo);
                        let last = (s.heights.index_at(hi) + 1).min(len);
                        for (i, it) in list.iter().enumerate().take(last).skip(first) {
                            let k = (self.key)(it);
                            if s.index_of.get(&k) != Some(&i) {
                                continue;
                            }
                            let need = match s.rows.get(&k) {
                                Some(r) => r.sig.with_untracked(|cur| cur != it),
                                None => true,
                            };
                            visible.push((k, need.then(|| it.clone()), s.heights.prefix(i), None));
                        }
                        // The row with the focus stays mounted outside the range.
                        if let Some(fk) = focused_key.as_ref()
                            && let Some(&i) = s.index_of.get(fk)
                            && (i < first || i >= last)
                            && let Some(it) = list.get(i)
                        {
                            let need = match s.rows.get(fk) {
                                Some(r) => r.sig.with_untracked(|cur| cur != it),
                                None => true,
                            };
                            let top = s.heights.prefix(i);
                            let at = visible.partition_point(|v| v.2 < top);
                            visible.insert(at, (fk.clone(), need.then(|| it.clone()), top, None));
                        }
                        s.heights.total()
                    }
                }
            })
        });
        (total, visible)
    }
}

impl<K: kivact_runtime::ItemKey, T: kivact_runtime::PropValue> Seeker<K, T> {
    fn fixed_visible(
        &self,
        list: &[T],
        s: &VState<K, T>,
        h: f64,
        range: (f64, f64),
        focused_key: Option<&K>,
        visible: &mut Vec<(K, Option<T>, f64, Option<f64>)>,
    ) -> f64 {
        let (lo, hi) = range;
        let h = h.max(1.0);
        let first = (lo / h).floor() as usize;
        let last = ((hi / h).ceil() as usize).min(list.len());
        for (i, it) in list.iter().enumerate().take(last).skip(first) {
            let k = (self.key)(it);
            if s.index_of.get(&k) != Some(&i) {
                continue;
            }
            let need = match s.rows.get(&k) {
                Some(r) => r.sig.with_untracked(|cur| cur != it),
                None => true,
            };
            visible.push((k, need.then(|| it.clone()), i as f64 * h, Some(h)));
        }
        // The row with the focus stays mounted outside the range.
        if let Some(fk) = focused_key
            && let Some(&i) = s.index_of.get(fk)
            && (i < first || i >= last)
            && let Some(it) = list.get(i)
        {
            let need = match s.rows.get(fk) {
                Some(r) => r.sig.with_untracked(|cur| cur != it),
                None => true,
            };
            let top = i as f64 * h;
            let at = visible.partition_point(|v| v.2 < top);
            visible.insert(at, (fk.clone(), need.then(|| it.clone()), top, Some(h)));
        }
        list.len() as f64 * h
    }
}
