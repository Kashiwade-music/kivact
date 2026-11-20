//! Role: Mounting a virtual list: its scroll box, spacer and bindings.

use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;
use kivact_runtime::driver::{EffectKind, Runtime};
use kivact_runtime::{ErrorKind, Prop as InputProp, Signal, Trigger, UiError, untrack};
use kivact_types::{LayoutProps, NodeId, Overflow, Position};

use super::builder::VirtualList;
use super::heights::Heights;
use super::rows::{apply_visible, px};
use super::seeker::Seeker;
use super::state::{HeldState, Seek, UNCONSTRAINED_ROWS, UNREPORTED_HEIGHT, VState, Viewport};
use crate::view::descriptor::{IntoView, Mcx, View, view_fn};
use crate::view::flow::list::RowFn;
use crate::view::mount::mount_host_with;
use crate::view::state::ViewState;

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
            let measured_version = cx.owner.trigger(at);
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
                heights: Heights::empty(),
                index_of: HashMap::new(),
                indexed: 0,
                seek: Seek::Idle,
                held: HeldState {
                    value: None,
                    changed: cx.owner.trigger(at),
                },
            }));
            // Which row holds the focus changes rarely: only then does the
            // visible range run again, not at every focus move.
            if keep_focused {
                bind_held(cx, spacer, &state, at);
            }
            let seeker = Rc::new(Seeker {
                items: items.clone(),
                key: key.clone(),
                height,
                state: state.clone(),
                viewport,
                outer,
            });
            bind_index(cx, seeker.clone(), at);
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

/// Track the row that holds the keyboard focus (`keep_focused`).
fn bind_held<K: kivact_runtime::ItemKey, T: 'static>(
    cx: &Mcx<'_>,
    spacer: NodeId,
    state: &Rc<RefCell<VState<K, T>>>,
    at: &'static Location<'static>,
) {
    let keys = ViewState::of(cx.rt)
        .focus_keys(cx.window)
        .expect("invariant: the list's window is open");
    let weak = cx.weak();
    let st = state.clone();
    cx.effect(EffectKind::Internal, at, move || {
        let node = keys.focus.get().map(|key| key.target);
        let Some(rt) = weak.upgrade() else { return };
        let held = node.and_then(|node| focused_row(&rt, spacer, node.node, &st));
        st.borrow_mut().held.set(held);
    });
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

/// The list changed: index the keys (the first occurrence wins), report
/// duplicates once, rebuild the row heights and forget the measurements
/// of keys that left. Its effect is made before `bind_visible`'s, so it
/// runs first in the same change.
fn bind_index<K: kivact_runtime::ItemKey, T: kivact_runtime::PropValue>(
    cx: &Mcx<'_>,
    seeker: Rc<Seeker<K, T>>,
    at: &'static Location<'static>,
) {
    let dup_owner = cx.owner;
    cx.effect(EffectKind::Internal, at, move || {
        let dups = seeker.reindex();
        if dups.is_empty() {
            return;
        }
        if let Some(rt) = Runtime::get(dup_owner.app()) {
            for h in dups {
                rt.report(
                    None,
                    UiError::new(
                        ErrorKind::DuplicateKey { key_hash: h },
                        "duplicate virtual_list key; later item dropped",
                        Some(at),
                    ),
                );
            }
        }
    });
}

fn bind_visible<K: kivact_runtime::ItemKey, T: kivact_runtime::PropValue>(
    cx: &Mcx<'_>,
    seeker: Rc<Seeker<K, T>>,
    row: RowFn<K, T>,
    spacer: NodeId,
    overscan: f64,
    measured_version: Trigger,
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
                let (total, visible) = range_seeker.visible(lo, hi, focused_key);
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
