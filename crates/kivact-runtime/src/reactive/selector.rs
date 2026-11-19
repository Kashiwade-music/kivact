//! Role: `Selector`: O(1) "is this key selected" reads (Solid `createSelector`).
//! One internal effect follows the source; each key read so far has its own
//! value-less node per reading owner, and a change notifies only the nodes of
//! the previous and the next key. An owner's disposal releases the key nodes
//! its computations read. A key node is removed when its last reader stops
//! reading it (12 章 ST-04).

use crate::reactive::AppId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::panic::Location;
use std::rc::Rc;

use super::props::{IntoValue, Prop};
use crate::reactive::arena::Key;
use crate::reactive::data::{EffectKind, Runtime};
use crate::reactive::registry::{runtime, try_runtime};

/// What the selector's effect and its readers share.
struct SelectorCache<K> {
    current: Option<K>,
    /// The value-less node of each key for each owner that read it: an
    /// owner's disposal releases its nodes.
    nodes: HashMap<K, Vec<(Key, Key)>>,
}
pub(crate) struct SelectorState<K> {
    owner: Key,
    effect: Key,
    cache: Rc<RefCell<SelectorCache<K>>>,
}

/// O(1) "is this key selected" reads (Solid `createSelector`).
pub struct Selector<K: 'static> {
    app: AppId,
    key: Key,
    _m: PhantomData<(fn() -> K, crate::reactive::UiThreadOnly)>,
}
impl<K: 'static> Clone for Selector<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: 'static> Copy for Selector<K> {}

/// Create a selector following `src` under `owner` (the caller has checked
/// that setup is running).
pub(crate) fn create_selector<K: crate::ItemKey>(
    rt: &Runtime,
    owner: Key,
    src: Prop<Option<K>>,
    at: &'static Location<'static>,
) -> Selector<K> {
    let cache = Rc::new(RefCell::new(SelectorCache::<K> {
        current: None,
        nodes: HashMap::new(),
    }));
    let app = rt.id;
    let weak = Rc::downgrade(&cache);
    let effect = rt.create_effect(
        owner,
        EffectKind::Internal,
        Box::new(move || {
            let Some(cache) = weak.upgrade() else { return };
            let Some(rt) = try_runtime(app) else { return };
            let next = {
                let _pure = rt.pure_section();
                src.get()
            };
            let prev = std::mem::replace(&mut cache.borrow_mut().current, next.clone());
            if prev == next {
                return;
            }
            let targets: Vec<Key> = {
                let cache = cache.borrow();
                let nodes = &cache.nodes;
                [prev, next]
                    .iter()
                    .flatten()
                    .filter_map(|k| nodes.get(k))
                    .flat_map(|list| list.iter().map(|(_, node)| *node))
                    .filter(|node| rt.node_alive(*node))
                    .collect()
            };
            let _g = rt.untracked_frame();
            for t in targets {
                rt.notify_subscribers(t);
            }
        }),
        at,
    );
    let state = Rc::new(SelectorState {
        owner,
        effect,
        cache,
    });
    let key = rt.insert_object(owner, state, "Selector", None, at);
    Selector {
        app,
        key,
        _m: PhantomData,
    }
}

/// A selector following `source` (Solid `createSelector`): `is(&k)` notifies
/// only the readers of the keys whose answer changed. Setup only.
///
/// ```ignore
/// let selected = selector(selected_id);
/// row_style(move || selected.is(&id))
/// ```
#[track_caller]
pub fn selector<K: crate::ItemKey>(source: impl IntoValue<Option<K>>) -> Selector<K> {
    let at = Location::caller();
    let (rt, owner) = crate::reactive::scope::setup_scope("selector", at);
    create_selector(&rt, owner, source.into_prop(), at)
}

impl<K: crate::ItemKey> Selector<K> {
    /// Whether `k` is the selected key; subscribes the current computation
    /// to changes of that answer only. The node is owned by the owner of that
    /// computation, so unmounting it releases the node.
    #[track_caller]
    pub fn is(&self, k: &K) -> bool {
        let at = Location::caller();
        let rt = runtime(self.app, at);
        let state = rt.object_or_panic::<SelectorState<K>>(self.key, "Selector", at);
        // The source was written earlier in this turn and the selector has
        // not caught up yet: bring it up to date first.
        if rt.needs_run(state.effect) {
            rt.run_effect(state.effect);
        }
        // Read outside a computation: the answer only, nothing to subscribe.
        if !rt.is_tracking() {
            return state.cache.borrow().current.as_ref() == Some(k);
        }
        // A reader outside a live owner (a disposing frame) reads the
        // selector's own owner: its node lives as long as the selector.
        let reader = rt
            .current_owner()
            .filter(|owner| rt.owner_live(*owner))
            .unwrap_or(state.owner);
        let existing = state.cache.borrow().nodes.get(k).and_then(|nodes| {
            nodes
                .iter()
                .rev()
                .find(|(owner, _)| *owner == reader)
                .map(|(_, node)| *node)
        });
        let node = match existing {
            Some(n) => n,
            None => {
                let n = rt.create_source(reader, (), at);
                state
                    .cache
                    .borrow_mut()
                    .nodes
                    .entry(k.clone())
                    .or_default()
                    .push((reader, n));
                // When its last reader stops reading it (a re-run reads another
                // key, or the reader is disposed), the node and its pair go.
                let cache = Rc::downgrade(&state.cache);
                let key = k.clone();
                rt.on_unsubscribed(
                    n,
                    Box::new(move |rt: &Runtime| {
                        if let Some(cache) = cache.upgrade() {
                            let mut cache = cache.borrow_mut();
                            if let Some(nodes) = cache.nodes.get_mut(&key) {
                                nodes.retain(|(_, node)| *node != n);
                                if nodes.is_empty() {
                                    cache.nodes.remove(&key);
                                }
                            }
                        }
                        drop(rt.remove_owned_node(n));
                    }),
                );
                n
            }
        };
        rt.read_node(node, true, at);
        state.cache.borrow().current.as_ref() == Some(k)
    }
}

#[cfg(test)]
mod tests {
    //! Role: A selector keeps no record of a reader that was unmounted.

    use std::panic::Location;
    use std::rc::Rc;

    use crate::reactive::data::{EffectKind, Runtime};
    use crate::reactive::host_types::Host;

    struct Quiet;

    impl Host for Quiet {
        fn wake(&self) {}
    }

    #[test]
    fn remounted_readers_do_not_accumulate_key_entries() {
        let rt = Runtime::register(
            Rc::new(Quiet),
            0,
            crate::driver::TaskState::new(None, crate::driver::ThreadWaker::new(|| {})),
        );
        let at = Location::caller();
        // The root owner takes the first owner slot, as `Runtime::new` makes it.
        assert_eq!(rt.create_owner(None, at), rt.root);
        rt.enter(|| {
            let root = rt.root();
            let source = root.signal(None::<u32>, at);
            let sel = root.with_setup(|| super::selector(source));
            for _ in 0..10 {
                let row = root.child(at);
                row.effect(
                    EffectKind::Internal,
                    move || {
                        let _ = sel.is(&1);
                    },
                    at,
                );
                row.dispose();
            }
            let row = root.child(at);
            row.effect(
                EffectKind::Internal,
                move || {
                    let _ = sel.is(&1);
                },
                at,
            );
            let state = rt.object_or_panic::<super::SelectorState<u32>>(sel.key, "Selector", at);
            let entries = state.cache.borrow().nodes.get(&1).map_or(0, Vec::len);
            assert_eq!(entries, 1, "only the live reader keeps an entry");
        });
        rt.shutdown();
    }

    #[test]
    fn a_reader_switching_keys_keeps_its_nodes_bounded() {
        let rt = Runtime::register(
            Rc::new(Quiet),
            0,
            crate::driver::TaskState::new(None, crate::driver::ThreadWaker::new(|| {})),
        );
        let at = Location::caller();
        assert_eq!(rt.create_owner(None, at), rt.root);
        rt.enter(|| {
            let root = rt.root();
            let source = root.signal(None::<u32>, at);
            let key = root.signal(0u32, at);
            let sel = root.with_setup(|| super::selector(source));
            root.effect(
                EffectKind::Internal,
                move || {
                    let _ = sel.is(&key.get());
                },
                at,
            );
            let before = rt.live_counts().reactive;
            for k in 1..=10_000u32 {
                key.set(k);
                rt.stabilize();
            }
            let after = rt.live_counts().reactive;
            assert!(
                after <= before,
                "one live reader switching keys keeps {before} nodes, not {after}"
            );
        });
        rt.shutdown();
    }
}
