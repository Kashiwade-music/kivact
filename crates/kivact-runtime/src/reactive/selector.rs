//! Role: `Selector`: O(1) "is this key selected" reads (Solid `createSelector`).
//! One internal effect follows the source; each key read so far has its own
//! value-less node per reading owner, and a change notifies only the nodes of
//! the previous and the next key. An owner's disposal releases the key nodes
//! its computations read. Unread key nodes are pruned as the key set grows.

use crate::reactive::AppId;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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
    prune: PruneSchedule,
}
struct PruneSchedule {
    at: usize,
}
impl PruneSchedule {
    fn due(&self, len: usize) -> bool {
        len >= self.at
    }
    fn retained(&mut self, len: usize) {
        self.at = (len * 2).max(SELECTOR_PRUNE_MIN);
    }
}
pub(crate) struct SelectorState<K> {
    owner: Key,
    effect: Key,
    cache: Rc<RefCell<SelectorCache<K>>>,
}

const SELECTOR_PRUNE_MIN: usize = 64;

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
        prune: PruneSchedule {
            at: SELECTOR_PRUNE_MIN,
        },
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
        // A reader outside a live owner (setup, or a disposing frame) reads
        // the selector's own owner: its node lives as long as the selector.
        let reader = rt
            .current_owner()
            .filter(|owner| rt.owner_live(*owner))
            .unwrap_or(state.owner);
        let existing = state.cache.borrow().nodes.get(k).and_then(|nodes| {
            nodes
                .iter()
                .rev()
                .find(|(owner, node)| *owner == reader && rt.node_alive(*node))
                .map(|(_, node)| *node)
        });
        let node = match existing {
            Some(n) => n,
            None => {
                let n = rt.create_source(reader, (), at);
                let len = state.cache.borrow().nodes.len();
                if state.cache.borrow().prune.due(len) {
                    self.prune(&rt, &state);
                    let left = state.cache.borrow().nodes.len();
                    state.cache.borrow_mut().prune.retained(left);
                }
                state
                    .cache
                    .borrow_mut()
                    .nodes
                    .entry(k.clone())
                    .or_default()
                    .push((reader, n));
                n
            }
        };
        rt.read_node(node, true, at);
        state.cache.borrow().current.as_ref() == Some(k)
    }

    /// Remove key nodes nobody reads any more (bounded growth). Keys read by
    /// a computation that is still running have no subscription yet (it is
    /// linked when the run ends), so they are kept.
    fn prune(&self, rt: &Runtime, state: &SelectorState<K>) {
        let reading: HashSet<Key> = rt
            .collectors
            .borrow()
            .iter()
            .flat_map(|collector| collector.deps.iter().copied())
            .collect();
        let dead: Vec<(K, Key)> = {
            let cache = state.cache.borrow();
            let keys = &cache.nodes;
            let g = rt.graph.borrow();
            let mut dead = Vec::new();
            for (k, nodes) in keys.iter() {
                for (_, n) in nodes {
                    if !reading.contains(n) && g.get(*n).is_none_or(|x| x.subs().is_empty()) {
                        dead.push((k.clone(), *n));
                    }
                }
            }
            dead
        };
        for (k, n) in dead {
            {
                let mut cache = state.cache.borrow_mut();
                if let Some(nodes) = cache.nodes.get_mut(&k) {
                    nodes.retain(|(_, node)| *node != n);
                    if nodes.is_empty() {
                        cache.nodes.remove(&k);
                    }
                }
            }
            let removed = rt.remove_owned_node(n);
            drop(removed);
        }
    }
}
