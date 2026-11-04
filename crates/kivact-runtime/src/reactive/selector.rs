//! Role: `Selector`: O(1) "is this key selected" reads (Solid `createSelector`).
//! One internal effect follows the source; each key read so far has its own
//! value-less node, and a change notifies only the nodes of the previous and
//! the next key. Unread key nodes are pruned as the key set grows.

use crate::reactive::AppId;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::panic::Location;
use std::rc::Rc;

use super::props::{IntoValue, Prop};
use crate::reactive::arena::Key;
use crate::reactive::data::{EffectKind, Runtime};
use crate::reactive::registry::{runtime, try_runtime};

/// What the selector's effect and its readers share.
struct SelectorKeys<K> {
    current: RefCell<Option<K>>,
    nodes: RefCell<HashMap<K, Key>>,
}

pub(crate) struct SelectorState<K> {
    owner: Key,
    /// The internal effect following the source.
    effect: Key,
    keys: Rc<SelectorKeys<K>>,
    /// Key count at which unread key nodes are pruned next (doubles).
    prune_at: Cell<usize>,
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
pub(crate) fn create_selector<K: Eq + Hash + Clone + 'static>(
    rt: &Runtime,
    owner: Key,
    src: Prop<Option<K>>,
    at: &'static Location<'static>,
) -> Selector<K> {
    let keys = Rc::new(SelectorKeys::<K> {
        current: RefCell::new(None),
        nodes: RefCell::new(HashMap::new()),
    });
    let app = rt.id;
    let weak = Rc::downgrade(&keys);
    let effect = rt.create_effect(
        owner,
        EffectKind::Internal,
        Box::new(move || {
            let Some(keys) = weak.upgrade() else { return };
            let Some(rt) = try_runtime(app) else { return };
            let next = {
                let _pure = rt.pure_section();
                src.get()
            };
            let prev = keys.current.replace(next.clone());
            if prev == next {
                return;
            }
            let targets: Vec<Key> = {
                let nodes = keys.nodes.borrow();
                [prev, next]
                    .iter()
                    .flatten()
                    .filter_map(|k| nodes.get(k).copied())
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
        keys,
        prune_at: Cell::new(SELECTOR_PRUNE_MIN),
    });
    let key = rt.insert_object(owner, state, None, at);
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
pub fn selector<K: Eq + Hash + Clone + 'static>(source: impl IntoValue<Option<K>>) -> Selector<K> {
    let at = Location::caller();
    let (rt, owner) = crate::reactive::scope::setup_scope("selector", at);
    create_selector(&rt, owner, source.into_prop(), at)
}

impl<K: Eq + Hash + Clone + 'static> Selector<K> {
    /// Whether `k` is the selected key; subscribes the current computation
    /// to changes of that answer only.
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
        let existing = state.keys.nodes.borrow().get(k).copied();
        let node = match existing.filter(|n| rt.node_alive(*n)) {
            Some(n) => n,
            None => {
                let n = rt.create_source(state.owner, (), at);
                let len = state.keys.nodes.borrow().len();
                if len >= state.prune_at.get() {
                    self.prune(&rt, &state);
                    let left = state.keys.nodes.borrow().len();
                    state.prune_at.set((left * 2).max(SELECTOR_PRUNE_MIN));
                }
                state.keys.nodes.borrow_mut().insert(k.clone(), n);
                n
            }
        };
        rt.read_node(node, true, at);
        state.keys.current.borrow().as_ref() == Some(k)
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
            let keys = state.keys.nodes.borrow();
            let g = rt.graph.borrow();
            keys.iter()
                .filter(|(_, n)| {
                    !reading.contains(*n) && g.get(**n).is_none_or(|x| x.subs().is_empty())
                })
                .map(|(k, n)| (k.clone(), *n))
                .collect()
        };
        for (k, n) in dead {
            state.keys.nodes.borrow_mut().remove(&k);
            let removed = rt.remove_owned_node(n);
            drop(removed);
        }
    }
}
