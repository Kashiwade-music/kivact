//! Role: Virtualized list with viewport-based row mounting.
//!
//! `virtual_list`: a scroll box whose visible range is computed by the core
//! from `ScrollChanged` (offset, viewport). Only visible rows (plus overscan)
//! are mounted; each row is an absolutely positioned wrapper. The host needs
//! no virtualization-specific node kind. Each row is set up in its own
//! owner; the row builder receives the key and the row's `Value<T>`.
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

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::bridge::PropValue;
use kivact_wire::{Dimension, Edges, LayoutProps, NodeId, NodeType, Overflow, Point, Position};

use crate::view::boundary::mount_in;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::{IntoView, View, view_fn};
use crate::view::flow::list::{RowFn, key_hash, lis};
use crate::view::mount::mount_host_with;
use crate::view::node::{Content, Node, NodeBuilder};
use crate::view::state::ViewState;
use kivact_reactive::{
    EffectKind, ErrorKind, IntoValue, Owner, Prop as InputProp, Runtime, Signal, Trigger, UiError,
    Value, untrack,
};
use kivact_wire::Role;

/// Row height policy of a virtual list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RowHeight {
    /// Every row has this height (logical px).
    Fixed(f32),
    /// Rows are measured by the host (`LayoutChanged`) and cached per key;
    /// `estimate` is used until a row has been measured.
    Measured { estimate: f32 },
}

/// Virtualized keyed list. Built by [`virtual_list`].
pub struct VirtualList<T: 'static, K: 'static> {
    items: InputProp<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    row: RowFn<K, T>,
    height: RowHeight,
    overscan: f32,
    scroll_to: InputProp<Option<K>>,
    keep_focused: bool,
    /// Holds the modifiers (`NodeBuilder`); its content and children are unused.
    root: Node,
    at: &'static Location<'static>,
}

/// A virtualized keyed list: only the rows in (or near) the viewport are
/// mounted, each as `row(key, item)` in its own owner.
///
/// ```ignore
/// virtual_list(rows, |r| r.id, log_row).row_height(RowHeight::Fixed(22.0))
/// ```
#[track_caller]
pub fn virtual_list<T, K, V>(
    items: impl IntoValue<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    row: impl Fn(K, Value<T>) -> V + 'static,
) -> VirtualList<T, K>
where
    T: Clone + PartialEq + 'static,
    K: Eq + Hash + Clone + 'static,
    V: IntoView,
{
    VirtualList {
        items: items.into_prop(),
        key: Rc::new(key),
        row: Rc::new(move |k, v| row(k, v).into_view()),
        height: RowHeight::Fixed(24.0),
        overscan: 200.0,
        scroll_to: kivact_reactive::Prop::constant(None),
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
    pub fn overscan(mut self, px: f32) -> Self {
        self.overscan = px.max(0.0);
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
    pub fn keep_focused(mut self, keep: bool) -> Self {
        self.keep_focused = keep;
        self
    }
}

impl<T: 'static, K: 'static> NodeBuilder for VirtualList<T, K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

struct VRow<T: 'static> {
    owner: Owner,
    wrapper: NodeId,
    sig: Signal<T>,
    top: f32,
}

struct VState<K: 'static, T: 'static> {
    rows: HashMap<K, VRow<T>>,
    measured: HashMap<K, f32>,
    /// The row `scroll_to` named, until the host has scrolled it into view.
    pending: Option<K>,
    /// The row that holds the keyboard focus (`keep_focused`).
    held: Option<K>,
}

/// Rows mounted at once above which an unconstrained viewport is reported.
const UNCONSTRAINED_ROWS: usize = 100;

fn px(v: f32) -> Dimension {
    Dimension::Px(v)
}

fn wrapper_layout(top: f32, height: Option<f32>) -> LayoutProps {
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

impl<T: Clone + PartialEq + 'static, K: Eq + Hash + Clone + 'static> IntoView
    for VirtualList<T, K>
{
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
            // `.at(..)` relocates the spacer and row wrappers too.
            let at = common.at.unwrap_or(at);
            // viewport (offset_y, height)
            let viewport: Signal<(f32, f32)> = cx.owner.signal((0.0f32, 0.0f32), at);
            let measured_version = cx.owner.trigger(at);
            common.handlers.scroll = Some(Rc::new(move |s| {
                viewport.set((s.offset.y, s.viewport.height))
            }));
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
                pending: None,
                held: None,
            }));
            // Which row holds the focus changes rarely: only then does the
            // visible range run again, not at every focus move.
            let held_version = cx.owner.trigger(at);
            if keep_focused && let Some(keys) = ViewState::of(cx.rt).focus_keys() {
                let weak = cx.weak();
                let st = state.clone();
                cx.effect(EffectKind::Internal, at, move || {
                    let node = keys.node.get();
                    let Some(rt) = weak.upgrade() else { return };
                    let held = node.and_then(|node| focused_row(&rt, spacer, node.node, &st));
                    if st.borrow().held != held {
                        st.borrow_mut().held = held;
                        held_version.notify();
                    }
                });
            }
            let seek = Rc::new(Seek {
                items: items.clone(),
                key: key.clone(),
                height,
                state: state.clone(),
                viewport,
                outer,
                issued: Cell::new(None),
                steps: Cell::new(0),
            });
            let weak = cx.weak();
            let parent = cx.owner;
            let st = state.clone();
            let warned = std::cell::Cell::new(false);
            let range_seek = seek.clone();
            cx.effect(EffectKind::Internal, at, move || {
                let (offset, reported_vh) = viewport.get();
                measured_version.track();
                let Some(rt) = weak.upgrade() else { return };
                let vh = if reported_vh <= 0.0 {
                    600.0
                } else {
                    reported_vh
                };
                held_version.track();
                let focused_key = st.borrow().held.clone();
                let lo = (offset - overscan).max(0.0);
                let hi = offset + vh + overscan;
                // Visible slice: (key, item clone if new or changed, top, height)
                let mut visible: Vec<(K, Option<T>, f32, Option<f32>)> = Vec::new();
                let mut dups = Vec::new();
                // Items and keys only compute values: pure.
                let total = kivact_reactive::pure(|| items.with(|list| {
                    let s = st.borrow();
                    match height {
                        RowHeight::Fixed(h) => {
                            let h = h.max(1.0);
                            let first = (lo / h).floor() as usize;
                            let last = ((hi / h).ceil() as usize).min(list.len());
                            let mut seen = std::collections::HashSet::new();
                            for (i, it) in list.iter().enumerate().take(last).skip(first) {
                                let k = key(it);
                                if !seen.insert(k.clone()) {
                                    dups.push(key_hash(&k));
                                    continue;
                                }
                                let need = match s.rows.get(&k) {
                                    Some(r) => r.sig.with_untracked(|cur| cur != it),
                                    None => true,
                                };
                                visible.push((k, need.then(|| it.clone()), i as f32 * h, Some(h)));
                            }
                            // The row with the focus stays mounted outside the range.
                            if let Some(fk) = focused_key.as_ref().filter(|fk| !seen.contains(*fk))
                                && let Some((i, it)) =
                                    list.iter().enumerate().find(|(_, it)| key(it) == *fk)
                            {
                                let need = match s.rows.get(fk) {
                                    Some(r) => r.sig.with_untracked(|cur| cur != it),
                                    None => true,
                                };
                                let top = i as f32 * h;
                                let at = visible.partition_point(|v| v.2 < top);
                                visible.insert(
                                    at,
                                    (fk.clone(), need.then(|| it.clone()), top, Some(h)),
                                );
                            }
                            list.len() as f32 * h
                        }
                        RowHeight::Measured { estimate } => {
                            let mut y = 0.0f32;
                            let mut seen = std::collections::HashSet::new();
                            for it in list {
                                let k = key(it);
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
                            let mut sm = st.borrow_mut();
                            if sm.measured.len() > list.len() + UNCONSTRAINED_ROWS {
                                let live: std::collections::HashSet<K> = list.iter().map(|it| key(it)).collect();
                                sm.measured.retain(|k, _| live.contains(k));
                            }
                            y
                        }
                    }
                }));
                // The host reported a viewport as tall as the content: the
                // list is not height-constrained and every row is mounted
                // (typically a flex ancestor without `min_height(0)`).
                if reported_vh > 0.0
                    && reported_vh + 0.5 >= total
                    && visible.len() >= UNCONSTRAINED_ROWS
                    && !warned.replace(true)
                {
                    let msg = format!(
                        "virtual_list at {at}: viewport ({reported_vh:.0}px) is as tall as its                          content ({total:.0}px), so all {} rows are mounted; give the list a                          bounded height (e.g. `min_height(0)` on flex ancestors or a fixed height)",
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
                    spacer,
                    &st,
                    visible,
                    &*row,
                    measured_version,
                    at,
                );
                range_seek.run(&rt);
            });
            // Scroll to the row `scroll_to` names, at mount and after each
            // change of the request.
            if scroll_to.fixed().is_none_or(|v| v.is_some()) {
                let weak = cx.weak();
                let last: RefCell<Option<K>> = RefCell::new(None);
                cx.effect(EffectKind::Internal, at, move || {
                    let request = scroll_to.get();
                    let Some(rt) = weak.upgrade() else { return };
                    if *last.borrow() == request {
                        return;
                    }
                    last.borrow_mut().clone_from(&request);
                    let Some(request) = request else { return };
                    seek.state.borrow_mut().pending = Some(request);
                    seek.issued.set(None);
                    seek.steps.set(0);
                    untrack(|| seek.run(&rt));
                });
            }
            outer
        })
    }
}

/// The mounted row that holds the keyboard focus, or has a node that does.
fn focused_row<K: Eq + Hash + Clone + 'static, T: 'static>(
    rt: &Runtime,
    spacer: NodeId,
    focused: NodeId,
    state: &RefCell<VState<K, T>>,
) -> Option<K> {
    let mut wrapper = focused;
    let tree = ViewState::tree(rt);
    // The row wrapper is the child of the spacer on the way up.
    loop {
        let parent = tree.get(wrapper)?.parent?;
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
fn reveal_offset(top: f32, height: f32, offset: f32, view: f32) -> f32 {
    if top < offset {
        top
    } else {
        offset + (top + height - offset - view).clamp(0.0, top - offset)
    }
}

/// What carries out `scroll_to`: the pending request becomes a scroll of the
/// list to the position of the row and, once the row is mounted, a scroll of
/// the host that brings the row into view.
struct Seek<K: 'static, T: 'static> {
    items: InputProp<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    height: RowHeight,
    state: Rc<RefCell<VState<K, T>>>,
    viewport: Signal<(f32, f32)>,
    outer: NodeId,
    /// The last scroll the list was asked for in this request: the offset
    /// it was at and the one asked for.
    issued: Cell<Option<(f32, f32)>>,
    /// How many scrolls were asked for in this request.
    steps: Cell<u8>,
}

/// Scrolls one request may take before it gives up (a `Measured` list whose
/// rows keep changing height under the target).
const MAX_STEPS: u8 = 8;

impl<K: Eq + Hash + Clone + 'static, T: Clone + 'static> Seek<K, T> {
    /// Advance the pending request, if any.
    fn run(&self, rt: &Rc<Runtime>) {
        let Some(want) = self.state.borrow().pending.clone() else {
            return;
        };
        if !ViewState::tree(rt).is_live(self.outer) {
            return;
        }
        // The row is mounted: the host can bring it into view exactly.
        let mounted = self.state.borrow().rows.get(&want).map(|r| r.wrapper);
        if let Some(wrapper) = mounted.filter(|w| ViewState::tree(rt).is_live(*w)) {
            self.finish();
            ViewState::tree(rt).engine().scroll_into_view(wrapper);
            rt.request_turn();
            return;
        }
        let (offset, view) = self.viewport.get_untracked();
        // The host has not reported a viewport yet.
        if view <= 0.0 {
            return;
        }
        if let Some((from, to)) = self.issued.get() {
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
        let place = kivact_reactive::pure(|| {
            self.items.with(|list| {
                let s = self.state.borrow();
                let mut found = None;
                let mut y = 0.0f32;
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
        if target == offset || self.steps.get() >= MAX_STEPS {
            self.finish();
            return;
        }
        self.steps.set(self.steps.get() + 1);
        self.issued.set(Some((offset, target)));
        ViewState::tree(rt)
            .engine()
            .scroll(self.outer, Point { x: 0.0, y: target }, false, false);
        rt.request_turn();
    }

    fn finish(&self) {
        self.state.borrow_mut().pending = None;
        self.issued.set(None);
        self.steps.set(0);
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "internal mount helper; the arguments are the list's parts"
)]
fn apply_visible<K: Eq + Hash + Clone + 'static, T: Clone + 'static>(
    rt: &Rc<Runtime>,
    parent: Owner,
    spacer: NodeId,
    state: &Rc<RefCell<VState<K, T>>>,
    visible: Vec<(K, Option<T>, f32, Option<f32>)>,
    row: &dyn Fn(K, Value<T>) -> View,
    measured_version: Trigger,
    at: &'static Location<'static>,
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
                let cx = Mcx { rt, owner };
                let wrapper = cx.host(NodeType::Box);
                cx.update(wrapper, PropValue::Layout(wrapper_layout(top, h)));
                if h.is_none() {
                    let st = Rc::downgrade(state);
                    let key = k.clone();
                    let mut tree = ViewState::tree_mut(rt);
                    tree.set_observe(wrapper, true);
                    if let Some(hd) = tree.handlers_mut(wrapper) {
                        hd.layout = Some(Rc::new(move |_, size| {
                            let Some(st) = st.upgrade() else { return };
                            let prev = st.borrow_mut().measured.insert(key.clone(), size.height);
                            if prev != Some(size.height) {
                                measured_version.notify();
                            }
                        }));
                    }
                }
                let content = mount_in(rt, owner, || row(k.clone(), sig.into_value()));
                cx.append(wrapper, content);
                if let Some(n) = ViewState::tree_mut(rt).get_mut(wrapper) {
                    n.parent = Some(spacer);
                }
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
    let mut tree = ViewState::tree_mut(rt);
    let old: Vec<NodeId> = tree
        .get(spacer)
        .map(|n| n.children.clone())
        .unwrap_or_default();
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
    let live = tree.is_live(spacer);
    tree.set_children(spacer, order.clone());
    if !live {
        return;
    }
    let mut next: Option<NodeId> = None;
    for i in (0..order.len()).rev() {
        let w = order[i];
        if fresh[i] {
            tree.attach_writes_before(w, next);
        } else if !stable[i] {
            tree.move_writes_before(w, next);
        }
        next = Some(w);
    }
}
