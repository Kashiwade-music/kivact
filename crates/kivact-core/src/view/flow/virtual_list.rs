//! Role: Virtualized list with viewport-based row mounting.
//!
//! `virtual_list`: a scroll box whose visible range is computed by the core
//! from `ScrollChanged` (offset, viewport). Only visible rows (plus overscan)
//! are mounted; each row is an absolutely positioned wrapper. The host needs
//! no virtualization-specific node kind. Each row is set up in its own
//! owner; the row builder receives the list source's row handle
//! (`ListSource::Row`), as `for_each` does: a `Value<T>`, or a `Binding<T>`
//! that writes back for a `Signal` or `Binding` source.
//!
//! The list is a scroll box, spacers and absolutely positioned rows; core
//! computes the visible range (±200 px of overscan by default).
//!
//! Invariants:
//! - `scroll_to(Option<key>)` acts each time the value becomes `Some`: it
//!   computes the row's position from the row heights (`Fixed`, or the
//!   measured heights and the estimate), moves the list with
//!   `Writer::scroll_to`, and finishes with `ScrollIntoView` once the row
//!   is mounted, at most 8 steps per request. `List::scroll_with` hands the
//!   target key to `scroll_to`, and a row that is not mounted takes the
//!   focus when it mounts.
//! - `keep_focused` (default `true`) keeps the row holding the focus (found
//!   by walking up from the focused node to a child of the spacer) mounted
//!   at its absolute position outside the range. An effect that reads only
//!   the focus source decides that row, and the range is computed again
//!   only when the row changes.
//! - Mounting 100 rows or more under an unconstrained size is warned
//!   about.

use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;
use kivact_types::{Dimension, Edges, LayoutProps, NodeId, Overflow, Point, Position};

use crate::view::boundary::mount_in;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::{IntoView, View, view_fn};
use crate::view::element_data::{Content, Element, Node};
use crate::view::flow::list::{ListInput, ListSource, RowFn, key_hash, lis, source_row};
use crate::view::mount::mount_host_with;
use crate::view::state::ViewState;
use kivact_runtime::{
    EffectKind, ErrorKind, IntoValue, Notifier, Owner, Prop as InputProp, Runtime, Signal, UiError,
    untrack,
};
use kivact_types::Role;

/// Row height policy of a virtual list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RowHeight {
    /// Every row has this height (logical px).
    Fixed(f64),
    /// Rows are measured by the host (`LayoutChanged`) and cached per key;
    /// `estimate` is used until a row has been measured.
    Measured { estimate: f64 },
}

/// Virtualized keyed list. Built by [`virtual_list`].
pub struct VirtualList<T: 'static, K: 'static> {
    items: InputProp<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    row: RowFn<K, T>,
    height: RowHeight,
    overscan: f64,
    scroll_to: InputProp<Option<K>>,
    keep_focused: bool,
    /// Holds the modifiers (`Element`); its content and children are unused.
    root: Node,
    at: &'static Location<'static>,
}

/// A virtualized keyed list: only the rows in (or near) the viewport are
/// mounted, each as `row(item)` in its own owner. `item` is the source's
/// row handle ([`ListSource::Row`]), the same as for
/// [`for_each`](crate::view::for_each): a `Value<T>`, or a `Binding<T>`
/// that writes back into a `Signal`/`Binding` source.
///
/// ```ignore
/// fn log_row(entry: Value<Entry>) -> impl IntoView { .. }
/// virtual_list(rows, |r| r.id, log_row).row_height(RowHeight::Fixed(22.0))
/// ```
#[track_caller]
pub fn virtual_list<T, K, V, S>(
    items: S,
    key: impl Fn(&T) -> K + 'static,
    row: impl Fn(S::Row) -> V + 'static,
) -> VirtualList<T, K>
where
    T: kivact_runtime::PropValue,
    K: kivact_runtime::ItemKey,
    V: IntoView,
    S: ListSource<T>,
{
    let ListInput {
        items,
        row: make_row,
    } = items.into_list_source();
    let key = Rc::new(key);
    VirtualList {
        items,
        row: source_row(key.clone(), make_row, row),
        key,
        height: RowHeight::Fixed(24.0),
        overscan: 200.0,
        scroll_to: kivact_runtime::Prop::constant(None),
        keep_focused: true,
        root: Node::new(Content::Box, None).role(Role::List),
        at: Location::caller(),
    }
}

impl<T: 'static, K: 'static> VirtualList<T, K> {
    pub fn row_height(mut self, h: RowHeight) -> Self {
        self.height = h;
        self
    }

    /// Extra pixels mounted above and below the viewport (default 200).
    pub fn overscan(mut self, px: impl Into<f64>) -> Self {
        self.overscan = px.into().max(0.0);
        self
    }

    /// Scroll so that the row `request` names is in view, each time the
    /// value changes to a `Some(key)`: another key, or the same key after a
    /// `None`, scrolls again; an unchanged value does not. The scroll is the
    /// smallest one that brings the row into the viewport, as
    /// [`NodeRef::scroll_into_view`](crate::view::NodeRef::scroll_into_view)
    /// does, so a row already in view does not move the list. A key that is
    /// not in the items does nothing.
    ///
    /// The row need not be mounted. Its position comes from the row height
    /// (`Fixed`, or for `Measured` the measured height of the rows above and
    /// the `estimate` for those never measured), the list scrolls there, and
    /// once the row is mounted the host scrolls it into view exactly. A
    /// `Measured` row far below rows that were never measured may therefore
    /// take more than one step to settle.
    ///
    /// ```ignore
    /// let target = signal(None::<u32>);
    /// virtual_list(rows, |r| r.id, log_row).scroll_to(target)
    /// // later: target.set(Some(4_200));
    /// ```
    pub fn scroll_to(mut self, request: impl IntoValue<Option<K>>) -> Self {
        self.scroll_to = request.into_prop();
        self
    }

    /// Whether the row that holds the keyboard focus (the row itself or a
    /// node inside it) stays mounted when it leaves the viewport and the
    /// overscan (default `true`). It stays at its position, outside the
    /// viewport and not hidden, so the focus is not lost while the list
    /// scrolls to another row or the user scrolls with the wheel. Once the
    /// focus has left it, the next update of the visible range unmounts the
    /// row as usual.
    pub fn release_offscreen_focus(mut self) -> Self {
        self.keep_focused = false;
        self
    }
}

impl<T: kivact_runtime::PropValue, K: kivact_runtime::ItemKey> Element for VirtualList<T, K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

struct VRow<T: 'static> {
    owner: Owner,
    wrapper: NodeId,
    sig: Signal<T>,
    top: f64,
}

/// What the host reported of the list's scroll box.
#[derive(Clone, Copy, PartialEq)]
struct Viewport {
    offset: f64,
    height: f64,
}

/// The viewport height the visible range assumes until the host reports one.
const UNREPORTED_HEIGHT: f64 = 600.0;

/// The `scroll_to` request in progress.
enum Seek<K> {
    Idle,
    /// The row `key` is to be scrolled into view, until the host has done it.
    Pending {
        key: K,
        issued: Option<Issued>,
        /// How many scrolls were asked for in this request.
        steps: u8,
    },
}

/// The last scroll the list asked for in a request: the offset it was at and the one it asked for.
#[derive(Clone, Copy)]
struct Issued {
    from: f64,
    to: f64,
}

struct VState<K: 'static, T: 'static> {
    rows: HashMap<K, VRow<T>>,
    measured: HashMap<K, f64>,
    /// The `scroll_to` request in progress.
    seek: Seek<K>,
    /// The row that holds the keyboard focus (`keep_focused`).
    held: HeldState<K>,
}

/// Rows mounted at once above which an unconstrained viewport is reported.
const UNCONSTRAINED_ROWS: usize = 100;

fn px(v: f64) -> Dimension {
    Dimension::Px(v)
}

fn wrapper_layout(top: f64, height: Option<f64>) -> LayoutProps {
    LayoutProps {
        position: Position::Absolute,
        inset: Edges {
            top: px(top),
            left: px(0.0),
            right: px(0.0),
            bottom: Dimension::Auto,
        },
        height: height.map(px).unwrap_or(Dimension::Auto),
        ..LayoutProps::default()
    }
}

impl<T: kivact_runtime::PropValue, K: kivact_runtime::ItemKey> IntoView for VirtualList<T, K> {
    fn into_view(self) -> View {
        view_fn(move |cx| {
            let VirtualList {
                items,
                key,
                row,
                height,
                overscan,
                scroll_to,
                keep_focused,
                root,
                at,
            } = self;
            let mut common = root.into_modifiers(cx, "a virtual list", at);
            // `.located_at_caller()` relocates the spacer and row wrappers too.
            let at = common.at.unwrap_or(at);
            // What the host reported (`None` before the first report).
            let viewport: Signal<Option<Viewport>> = cx.owner.signal(None, at);
            let measured_version = cx.owner.notifier(at);
            common.handlers.observe_scroll(move |s| {
                viewport.set(Some(Viewport {
                    offset: s.offset.y,
                    height: s.viewport.height,
                }))
            });
            // The style resolves like every node's (state styles and
            // paint transitions included), but overflow is always a
            // vertical scroll.
            let outer = mount_host_with(
                cx,
                NodeType::Box,
                common,
                at,
                Some(|layout| layout.overflow_y = Overflow::Scroll),
            );
            let spacer = cx.host(NodeType::Box);
            cx.update(
                spacer,
                PropValue::Layout(LayoutProps {
                    position: Position::Relative,
                    ..LayoutProps::default()
                }),
            );
            cx.append(outer, spacer);
            let state: Rc<RefCell<VState<K, T>>> = Rc::new(RefCell::new(VState {
                rows: HashMap::new(),
                measured: HashMap::new(),
                seek: Seek::Idle,
                held: HeldState {
                    value: Held::Nothing,
                    changed: cx.owner.notifier(at),
                },
            }));
            // Which row holds the focus changes rarely: only then does the
            // visible range run again, not at every focus move.
            if keep_focused {
                let keys = ViewState::of(cx.rt)
                    .focus_keys(cx.window)
                    .expect("invariant: the list's window is open");
                let weak = cx.weak();
                let st = state.clone();
                cx.effect(EffectKind::Internal, at, move || {
                    let node = keys.node.get();
                    let Some(rt) = weak.upgrade() else { return };
                    let held = node.and_then(|node| focused_row(&rt, spacer, node.node, &st));
                    st.borrow_mut().held.set(held);
                });
            }
            let seeker = Rc::new(Seeker {
                items: items.clone(),
                key: key.clone(),
                height,
                state: state.clone(),
                viewport,
                outer,
            });
            bind_visible(
                cx,
                seeker.clone(),
                row,
                spacer,
                overscan,
                measured_version,
                at,
            );
            bind_scroll_to(cx, scroll_to, seeker, at);
            outer
        })
    }
}

/// The mounted row that holds the keyboard focus, or has a node that does.
fn focused_row<K: kivact_runtime::ItemKey, T: 'static>(
    rt: &Runtime,
    spacer: NodeId,
    focused: NodeId,
    state: &RefCell<VState<K, T>>,
) -> Option<K> {
    let mut wrapper = focused;
    let tree = ViewState::tree(rt);
    // The row wrapper is the child of the spacer on the way up.
    loop {
        let parent = tree.reader().parent(wrapper)?;
        if parent == spacer {
            break;
        }
        wrapper = parent;
    }
    state
        .borrow()
        .rows
        .iter()
        .find(|(_, row)| row.wrapper == wrapper)
        .map(|(key, _)| key.clone())
}

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

/// What carries out `scroll_to`: the pending request becomes a scroll of the
/// list to the position of the row and, once the row is mounted, a scroll of
/// the host that brings the row into view.
struct Seeker<K: 'static, T: 'static> {
    items: InputProp<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    height: RowHeight,
    state: Rc<RefCell<VState<K, T>>>,
    viewport: Signal<Option<Viewport>>,
    outer: NodeId,
}

/// Scrolls one request may take before it gives up (a `Measured` list whose
/// rows keep changing height under the target).
const MAX_STEPS: u8 = 8;

impl<K: kivact_runtime::ItemKey, T: Clone + 'static> Seeker<K, T> {
    /// Advance the pending request, if any.
    fn run(&self, rt: &Rc<Runtime>) {
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
                let mut found = None;
                let mut y = 0.0f64;
                for it in list {
                    let k = (self.key)(it);
                    let h = match self.height {
                        RowHeight::Fixed(h) => h.max(1.0),
                        RowHeight::Measured { estimate } => {
                            s.measured.get(&k).copied().unwrap_or(estimate).max(1.0)
                        }
                    };
                    if found.is_none() && k == want {
                        found = Some((y, h));
                    }
                    y += h;
                }
                found.map(|(top, h)| (top, h, y))
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

#[allow(
    clippy::too_many_arguments,
    reason = "internal mount helper; the arguments are the list's parts"
)]
fn apply_visible<K: kivact_runtime::ItemKey, T: Clone + 'static>(
    rt: &Rc<Runtime>,
    parent: Owner,
    window: kivact_types::WindowId,
    spacer: NodeId,
    state: &Rc<RefCell<VState<K, T>>>,
    visible: Vec<(K, Option<T>, f64, Option<f64>)>,
    row: &RowFn<K, T>,
    measured_version: Notifier,
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
                            let prev = st.borrow_mut().measured.insert(key.clone(), size.height);
                            if prev != Some(size.height) {
                                measured_version.notify();
                            }
                        });
                    }
                }
                let fragment = cx.fragment();
                let content = mount_in(rt, owner, window, || row.call((k.clone(), sig)));
                cx.append(fragment, content);
                cx.append(wrapper, fragment);
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

#[derive(PartialEq)]
enum Held<K> {
    Nothing,
    Holding { key: K },
}
struct HeldState<K> {
    value: Held<K>,
    changed: kivact_runtime::Notifier,
}
impl<K: PartialEq> HeldState<K> {
    fn key(&self) -> Option<&K> {
        match &self.value {
            Held::Nothing => None,
            Held::Holding { key } => Some(key),
        }
    }
    fn set(&mut self, key: Option<K>) {
        let next = match key {
            Some(key) => Held::Holding { key },
            None => Held::Nothing,
        };
        if self.value != next {
            self.value = next;
            self.changed.notify();
        }
    }
}

impl<K: kivact_runtime::ItemKey, T: kivact_runtime::PropValue> Seeker<K, T> {
    fn visible(
        &self,
        lo: f64,
        hi: f64,
        focused_key: Option<K>,
    ) -> (f64, Vec<(K, Option<T>, f64, Option<f64>)>, Vec<u64>) {
        let mut visible = Vec::new();
        let mut dups = Vec::new();
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
                        (&mut visible, &mut dups),
                    ),
                    RowHeight::Measured { estimate } => {
                        let mut y = 0.0f64;
                        let mut seen = std::collections::HashSet::new();
                        for it in list {
                            let k = (self.key)(it);
                            let h = s.measured.get(&k).copied().unwrap_or(estimate).max(1.0);
                            if (y + h >= lo && y <= hi) || focused_key.as_ref() == Some(&k) {
                                if !seen.insert(k.clone()) {
                                    dups.push(key_hash(&k));
                                } else {
                                    let need = match s.rows.get(&k) {
                                        Some(r) => r.sig.with_untracked(|cur| cur != it),
                                        None => true,
                                    };
                                    visible.push((k, need.then(|| it.clone()), y, None));
                                }
                            }
                            y += h;
                        }
                        // Forget measurements of keys that left the list
                        // (bounded cache).
                        drop(s);
                        let mut sm = self.state.borrow_mut();
                        if sm.measured.len() > list.len() + UNCONSTRAINED_ROWS {
                            let live: std::collections::HashSet<K> =
                                list.iter().map(|it| (self.key)(it)).collect();
                            sm.measured.retain(|k, _| live.contains(k));
                        }
                        y
                    }
                }
            })
        });
        (total, visible, dups)
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
        output: (&mut Vec<(K, Option<T>, f64, Option<f64>)>, &mut Vec<u64>),
    ) -> f64 {
        let (lo, hi) = range;
        let (visible, dups) = output;
        let h = h.max(1.0);
        let first = (lo / h).floor() as usize;
        let last = ((hi / h).ceil() as usize).min(list.len());
        let mut seen = std::collections::HashSet::new();
        for (i, it) in list.iter().enumerate().take(last).skip(first) {
            let k = (self.key)(it);
            if !seen.insert(k.clone()) {
                dups.push(key_hash(&k));
                continue;
            }
            let need = match s.rows.get(&k) {
                Some(r) => r.sig.with_untracked(|cur| cur != it),
                None => true,
            };
            visible.push((k, need.then(|| it.clone()), i as f64 * h, Some(h)));
        }
        // The row with the focus stays mounted outside the range.
        if let Some(fk) = focused_key.filter(|fk| !seen.contains(*fk))
            && let Some((i, it)) = list
                .iter()
                .enumerate()
                .find(|(_, it)| (self.key)(it) == *fk)
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

fn bind_visible<K: kivact_runtime::ItemKey, T: kivact_runtime::PropValue>(
    cx: &Mcx<'_>,
    seeker: Rc<Seeker<K, T>>,
    row: RowFn<K, T>,
    spacer: NodeId,
    overscan: f64,
    measured_version: Notifier,
    at: &'static Location<'static>,
) {
    let weak = cx.weak();
    let parent = cx.owner;
    let window = cx.window;
    let st = seeker.state.clone();
    let warned = kivact_types::cell::ReportOnce::new();
    let range_seeker = seeker.clone();
    cx.effect(EffectKind::Internal, at, move || {
                let reported = seeker.viewport.get();
                let (offset, vh) =
                    reported.map_or((0.0, UNREPORTED_HEIGHT), |v| (v.offset, v.height));
                measured_version.track();
                let Some(rt) = weak.upgrade() else { return };
                seeker.state.borrow().held.changed.track();
                let focused_key = st.borrow().held.key().cloned();
                let lo = (offset - overscan).max(0.0);
                let hi = offset + vh + overscan;
                // Visible slice: (key, item clone if new or changed, top, height)
                let (total, visible, dups) = range_seeker.visible(lo, hi, focused_key);
                // The host reported a viewport as tall as the content: the
                // list is not height-constrained and every row is mounted
                // (typically a flex ancestor without `min_height(0)`).
                if let Some(v) = reported
                    && v.height > 0.0
                    && v.height + 0.5 >= total
                    && visible.len() >= UNCONSTRAINED_ROWS
                    && warned.note()
                {
                    let msg = format!(
                        "virtual_list at {at}: viewport ({:.0}px) is as tall as its                          content ({total:.0}px), so all {} rows are mounted; give the list a                          bounded height (e.g. `min_height(0)` on flex ancestors or a fixed height)",
                        v.height,
                        visible.len()
                    );
                    rt.warn(msg);
                }
                for h in dups {
                    rt.report(None, UiError::new(
                        ErrorKind::DuplicateKey { key_hash: h },
                        "duplicate virtual_list key; later item dropped",
                        Some(at),
                    ));
                }
                ViewState::tree_mut(&rt).update(
                    spacer,
                    PropValue::Layout(LayoutProps {
                        height: px(total),
                        min_height: px(total),
                        shrink: 0.0,
                        position: Position::Relative,
                        ..LayoutProps::default()
                    }),
                    rt.running_effect(),
                );
                apply_visible(
                    &rt,
                    parent,
                    window,
                    spacer,
                    &st,
                    visible,
                    &row,
                    measured_version,
                    at,
                );
                range_seeker.run(&rt);
            });
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
        removed.commit(rt);
        r.owner.dispose();
    }
}

fn bind_scroll_to<K: kivact_runtime::ItemKey, T: kivact_runtime::PropValue>(
    cx: &Mcx<'_>,
    scroll_to: InputProp<Option<K>>,
    seeker: Rc<Seeker<K, T>>,
    at: &'static Location<'static>,
) {
    // Scroll to the row `scroll_to` names, at mount and after each
    // change of the request.
    if scroll_to.fixed().is_none_or(|v| v.is_some()) {
        let weak = cx.weak();
        let last = RefCell::new(kivact_types::cell::Sent::<Option<K>>::new());
        cx.effect(EffectKind::Internal, at, move || {
            let request = scroll_to.get();
            let Some(rt) = weak.upgrade() else { return };
            if last.borrow_mut().update(request.clone()).is_none() {
                return;
            }
            let Some(request) = request else { return };
            seeker.state.borrow_mut().seek = Seek::Pending {
                key: request,
                issued: None,
                steps: 0,
            };
            untrack(|| seeker.run(&rt));
        });
    }
}
