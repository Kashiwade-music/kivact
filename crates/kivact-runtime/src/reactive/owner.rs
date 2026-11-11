//! Role: Owner tree and hierarchical resource disposal.
//!
//! Owner tree: every node, object, cleanup and context belongs to an owner;
//! disposing an owner releases all of them (registration/cleanup pairs).
//! Disposal is iterative (no recursion, an explicit stack); per owner it
//! runs child owners, then cleanups (LIFO), user effects, render and
//! internal effects, memos, sources and finally the objects.
//!
//! Owners are made at every structural boundary: the root, a flow branch, a
//! list row, `provide`/`provide_theme` and `error_boundary` (in
//! `kivact-core`, all through `boundary::mount_in`). What a setup creates
//! (signals, effects, nodes, tasks, actions, contexts, ...) belongs to the
//! owner of the current setup frame. Every registration has its release
//! (event subscriptions, tasks, timers, frame requests, action queues,
//! contexts, node references).
//!
//! Invariants (G-01: nothing registered for a disposed component outlives
//! it):
//! - A registration on an owner that is being disposed or is gone is the
//!   runtime's to handle, never the caller's to check:
//!   - a child owner is created disposed (its key is a tombstone), so what
//!     is registered under it is handled as for a gone owner;
//!   - a cleanup runs at once;
//!   - a node or an object of an owner being disposed is released by the
//!     same disposal, before the owner is removed (a cleanup may still use
//!     a signal it creates); one of a gone owner is released at once;
//!   - a context or an `on_error` handler of an owner being disposed is
//!     dropped with it; one of a gone owner is dropped at once.
//! - Releasing members may register more (cleanups, `on_dispose` hooks):
//!   the disposal releases those in rounds. When they do not settle within
//!   [`RELEASE_ROUNDS`] it reports an `ErrorKind::Cycle` to the default
//!   sink and removes the owner first, so what is registered after that is
//!   released at once.

use std::any::{Any, TypeId};
use std::panic::Location;
use std::rc::Rc;

use super::arena::Tomb;
use crate::reactive::arena::Key;
use crate::reactive::data::{
    ConstEntry, EffectKind, Kind, NodeData, OwnerEntry, OwnerPhase, Runtime,
};
use crate::reactive::error::{ErrorKind, UiError, misuse_at, new_error};

/// Rounds a disposal releases what its members registered while being
/// released, before it reports a `Cycle`.
pub(crate) const RELEASE_ROUNDS: usize = 16;

/// What a registration on an owner meets.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerState {
    /// Alive and not being disposed.
    Live,
    /// Its disposal has started; it is still in the arena.
    Disposing,
    /// Disposed (or never existed).
    Gone,
}

impl Runtime {
    /// Create an owner under `parent` for the boundary called at `at`.
    /// Under a parent that is being disposed or is gone, the owner is
    /// created disposed: the key is a tombstone, and what is registered
    /// under it is released at once.
    pub(crate) fn create_owner(&self, parent: Option<Key>, at: &'static Location<'static>) -> Key {
        let state = parent.map_or(OwnerState::Live, |p| self.owner_state(p));
        let mut owners = self.owners.borrow_mut();
        let key = owners.insert(OwnerEntry {
            parent,
            parent_index: 0,
            children: Vec::new(),
            nodes: Vec::new(),
            extra: None,
            created_at: at,
            phase: if state == OwnerState::Live {
                OwnerPhase::Live
            } else {
                OwnerPhase::Disposing { round: 0 }
            },
        });
        if state != OwnerState::Live {
            let disposed_by = parent.and_then(|p| match owners.get(p) {
                Some(o) => Some(o.created_at),
                None => owners.tomb(p).and_then(|t| t.disposed_by),
            });
            // Nothing is registered yet: the removed owner holds no user value.
            owners.remove(
                key,
                Some(Tomb {
                    created_at: at,
                    disposed_by,
                }),
            );
            return key;
        }
        if let Some(p) = parent.and_then(|p| owners.get_mut(p)) {
            p.children.push(key);
            let i = p.children.len() as u32 - 1;
            if let Some(o) = owners.get_mut(key) {
                o.parent_index = i;
            }
        }
        key
    }

    pub(crate) fn owner_state(&self, key: Key) -> OwnerState {
        match self.owners.borrow().get(key) {
            Some(o) if matches!(o.phase, OwnerPhase::Disposing { .. }) => OwnerState::Disposing,
            Some(_) => OwnerState::Live,
            None => OwnerState::Gone,
        }
    }

    pub(crate) fn owner_alive(&self, key: Key) -> bool {
        self.owners.borrow().contains(key)
    }

    /// Alive and not being disposed: the component it owns is still shown.
    /// Registrations need no such check (the runtime handles those on an
    /// owner being disposed); it is for work that only makes sense while
    /// the component lives (delivering an event, restoring a focus).
    pub(crate) fn owner_live(&self, key: Key) -> bool {
        self.owner_state(key) == OwnerState::Live
    }

    /// Run `f` when `owner` is disposed (LIFO with its other cleanups). On
    /// an owner that is being disposed or is gone, `f` runs at once, in a
    /// handler frame of the owner.
    pub(crate) fn add_cleanup(&self, owner: Key, f: Box<dyn FnOnce()>) {
        let f = match self.owners.borrow_mut().get_mut(owner) {
            Some(o) if o.phase == OwnerPhase::Live => {
                o.extra_mut().cleanups.push(f);
                return;
            }
            _ => f,
        };
        let _g = self.handler_frame(Some(owner));
        f();
    }

    pub(crate) fn add_effect_cleanup(&self, effect: Key, f: Box<dyn FnOnce()>) {
        if let Some(NodeData::Effect(e) | NodeData::Watcher(e)) =
            self.graph.borrow_mut().get_mut(effect).map(|n| &mut n.data)
        {
            e.cleanups.push(f);
        }
    }

    /// Provide `value` as a context of `owner` (keyed by its type). The
    /// value of a gone owner is dropped at once.
    pub(crate) fn provide_context(&self, owner: Key, value: Rc<dyn Any>) {
        let tid = (*value).type_id();
        let replaced = match self.owners.borrow_mut().get_mut(owner) {
            Some(o) => {
                let contexts = &mut o.extra_mut().contexts;
                let old = contexts
                    .iter()
                    .position(|(t, _)| *t == tid)
                    .map(|i| contexts.remove(i));
                contexts.push((tid, value));
                old
            }
            None => Some((tid, value)),
        };
        // User values are dropped after the borrow.
        drop(replaced);
    }

    /// Nearest context of type `tid` from `owner` upwards.
    pub(crate) fn find_context(&self, owner: Key, tid: TypeId) -> Option<Rc<dyn Any>> {
        let owners = self.owners.borrow();
        let mut cur = Some(owner);
        while let Some(k) = cur {
            let o = owners.get(k)?;
            if let Some((_, v)) = o
                .extra
                .as_ref()
                .and_then(|x| x.contexts.iter().find(|(t, _)| *t == tid))
            {
                return Some(v.clone());
            }
            cur = o.parent;
        }
        None
    }

    /// Install `handler` as the `on_error` handler of `owner` (dropped at
    /// once when the owner is gone).
    pub(crate) fn set_error_handler(&self, owner: Key, handler: Rc<dyn Fn(UiError)>) {
        let replaced = match self.owners.borrow_mut().get_mut(owner) {
            Some(o) => o.extra_mut().on_error.replace(handler),
            None => Some(handler),
        };
        drop(replaced);
    }

    /// Dispose `root` and its whole subtree: children first, then cleanups
    /// (LIFO), effects, memos, sources and objects. User values are dropped
    /// after all runtime borrows are released.
    pub(crate) fn dispose_owner(&self, root: Key) {
        let order = {
            let owners = self.owners.borrow();
            if !owners.contains(root) {
                return;
            }
            let mut pre = Vec::new();
            let mut stack = vec![root];
            while let Some(k) = stack.pop() {
                pre.push(k);
                if let Some(o) = owners.get(k) {
                    stack.extend(o.children.iter().copied());
                }
            }
            pre.reverse();
            pre
        };
        {
            let mut owners = self.owners.borrow_mut();
            for k in &order {
                if let Some(o) = owners.get_mut(*k) {
                    o.phase = OwnerPhase::Disposing { round: 0 };
                }
            }
        }
        self.detach_owner(root);
        // Disposal runs user cleanups: untracked, never pure.
        let _g = self.handler_frame(Some(root));
        for key in order {
            self.dispose_one(key);
        }
    }

    fn detach_owner(&self, key: Key) {
        let mut owners = self.owners.borrow_mut();
        let Some((parent, index)) = owners.get(key).map(|o| (o.parent, o.parent_index as usize))
        else {
            return;
        };
        let Some(parent) = parent else { return };
        let moved = owners.get_mut(parent).and_then(|p| {
            if index < p.children.len() && p.children[index] == key {
                p.children.swap_remove(index);
                p.children.get(index).copied()
            } else {
                None
            }
        });
        if let Some(m) = moved.and_then(|m| owners.get_mut(m)) {
            m.parent_index = index as u32;
        }
        if let Some(o) = owners.get_mut(key) {
            o.parent = None;
        }
    }

    fn dispose_one(&self, key: Key) {
        let (cleanups, created_at) = match self.owners.borrow_mut().get_mut(key) {
            Some(o) => (
                o.extra
                    .as_deref_mut()
                    .map(|x| std::mem::take(&mut x.cleanups))
                    .unwrap_or_default(),
                o.created_at,
            ),
            None => return,
        };
        if !cleanups.is_empty() {
            let _g = self.handler_frame(Some(key));
            for c in cleanups.into_iter().rev() {
                c();
            }
        }
        let mut graveyard: Vec<Box<dyn Any>> = Vec::new();
        // Releasing members may register more (cleanups, `on_dispose`
        // hooks): release those too, in bounded rounds.
        loop {
            let round = match self.owners.borrow().get(key).map(|o| o.phase) {
                Some(OwnerPhase::Disposing { round }) => round,
                _ => return,
            };
            let (nodes, objects, consts) = match self.owners.borrow_mut().get_mut(key) {
                Some(o) => {
                    let (objects, consts) = o
                        .extra
                        .as_deref_mut()
                        .map(|x| {
                            (
                                std::mem::take(&mut x.objects),
                                std::mem::take(&mut x.consts),
                            )
                        })
                        .unwrap_or_default();
                    (std::mem::take(&mut o.nodes), objects, consts)
                }
                None => return,
            };
            if nodes.is_empty() && objects.is_empty() && consts.is_empty() {
                break;
            }
            if usize::from(round) == RELEASE_ROUNDS {
                self.default_sink(new_error(
                    ErrorKind::Cycle {
                        path: vec![created_at],
                    },
                    format!(
                        "disposing an owner did not settle within {RELEASE_ROUNDS} rounds: \
                         releasing its members kept registering more"
                    ),
                    Some(created_at),
                ));
                // Remove the owner first: what the last members register
                // now is released at once.
                let removed = self.remove_disposed_owner(key, created_at);
                self.dispose_members(key, created_at, nodes, objects, consts, &mut graveyard);
                drop(removed);
                drop(graveyard);
                return;
            }
            self.dispose_members(key, created_at, nodes, objects, consts, &mut graveyard);
            if let Some(o) = self.owners.borrow_mut().get_mut(key) {
                o.phase = OwnerPhase::Disposing { round: round + 1 };
            }
        }
        let removed = self.remove_disposed_owner(key, created_at);
        drop(removed);
        drop(graveyard);
    }

    fn remove_disposed_owner(
        &self,
        key: Key,
        created_at: &'static Location<'static>,
    ) -> Option<OwnerEntry> {
        self.owners.borrow_mut().remove(
            key,
            Some(Tomb {
                created_at,
                disposed_by: Some(created_at),
            }),
        )
    }

    /// Release node `key`, just registered on `owner`, which is gone.
    pub(crate) fn release_orphan_node(&self, key: Key, owner: Key) {
        let disposed_by = self.owners.borrow().tomb(owner).and_then(|t| t.disposed_by);
        let removed = self.remove_node(key, disposed_by);
        drop(removed);
    }

    /// Release object `key`, just registered on `owner`, which is gone
    /// (`on_dispose` runs first).
    pub(crate) fn release_orphan_object(&self, key: Key, owner: Key) {
        let disposed_by = self.owners.borrow().tomb(owner).and_then(|t| t.disposed_by);
        let removed = {
            let mut objects = self.objects.borrow_mut();
            let created_at = objects.get(key).map(|o| o.created_at);
            created_at.and_then(|created_at| {
                objects.remove(
                    key,
                    Some(Tomb {
                        created_at,
                        disposed_by,
                    }),
                )
            })
        };
        if let Some(mut obj) = removed {
            if let Some(f) = obj.on_dispose.take() {
                f(self);
            }
            drop(obj);
        }
    }

    fn dispose_members(
        &self,
        key: Key,
        created_at: &'static Location<'static>,
        nodes: Vec<Key>,
        objects: Vec<Key>,
        consts: Vec<Key>,
        graveyard: &mut Vec<Box<dyn Any>>,
    ) {
        // Effects first (their cleanups may read memos and sources), then memos, then sources.
        let rank = |k: &Key| -> u8 {
            match self.graph.borrow().get(*k).map(|n| n.kind()) {
                Some(Kind::Effect(EffectKind::User)) => 0,
                Some(Kind::Effect(_)) => 1,
                Some(Kind::Memo) => 2,
                _ => 3,
            }
        };
        let mut nodes = nodes;
        nodes.sort_by_key(|k| rank(k));
        for n in nodes {
            let cleanups = self
                .graph
                .borrow_mut()
                .get_mut(n)
                .and_then(|r| match &mut r.data {
                    NodeData::Effect(e) | NodeData::Watcher(e) => {
                        Some(std::mem::take(&mut e.cleanups))
                    }
                    _ => None,
                });
            if let Some(cleanups) = cleanups.filter(|c| !c.is_empty()) {
                let _g = self.handler_frame(Some(key));
                for c in cleanups.into_iter().rev() {
                    c();
                }
            }
            if let Some(node) = self.remove_node(n, Some(created_at)) {
                graveyard.push(Box::new(node));
            }
        }
        for o in objects {
            let removed = self.objects.borrow_mut().remove(
                o,
                Some(Tomb {
                    created_at,
                    disposed_by: Some(created_at),
                }),
            );
            if let Some(mut obj) = removed {
                if let Some(f) = obj.on_dispose.take() {
                    f(self);
                }
                graveyard.push(Box::new(obj.data));
            }
        }
        for c in consts {
            let at = self.consts.borrow().get(c).map(|entry| entry.created_at);
            let Some(at) = at else { continue };
            let removed = self.consts.borrow_mut().remove(
                c,
                Some(Tomb {
                    created_at: at,
                    disposed_by: Some(created_at),
                }),
            );
            if let Some(entry) = removed {
                graveyard.push(Box::new(entry.data));
            }
        }
    }

    /// Create a constant of `owner` holding `value`; on a gone owner it is
    /// released at once.
    pub(crate) fn create_const<T: 'static>(
        &self,
        owner: Key,
        value: T,
        at: &'static Location<'static>,
    ) -> Key {
        let key = self.consts.borrow_mut().insert(ConstEntry {
            created_at: at,
            data: Rc::new(value),
        });
        let registered = self
            .owners
            .borrow_mut()
            .get_mut(owner)
            .map(|o| o.extra_mut().consts.push(key))
            .is_some();
        if !registered {
            let disposed_by = self.owners.borrow().tomb(owner).and_then(|t| t.disposed_by);
            let removed = self.consts.borrow_mut().remove(
                key,
                Some(Tomb {
                    created_at: at,
                    disposed_by,
                }),
            );
            drop(removed);
        }
        key
    }

    /// The value of constant `key` as `Rc<T>`, if it is alive and has that type.
    pub(crate) fn const_value<T: 'static>(&self, key: Key) -> Option<Rc<T>> {
        let data = self.consts.borrow().get(key).map(|c| c.data.clone())?;
        data.downcast::<T>().ok()
    }

    /// Located `Disposed` misuse for a constant whose owner is gone.
    pub(crate) fn const_disposed_panic(&self, key: Key, at: &'static Location<'static>) -> ! {
        let tomb = self.consts.borrow().tomb(key);
        misuse_at(
            ErrorKind::Disposed {
                created_at: tomb.map(|t| t.created_at),
                disposed_by: tomb.and_then(|t| t.disposed_by),
            },
            "Value read after its owner was disposed",
            at,
        )
    }
}

#[cfg(test)]
mod tests {
    //! Role: Owner disposal releases every rare member.

    use std::cell::RefCell;
    use std::panic::Location;
    use std::rc::Rc;

    use crate::reactive::AppId;
    use crate::reactive::data::Runtime;
    use crate::reactive::error::UiError;
    use crate::reactive::host_types::Host;

    struct Quiet;

    impl Host for Quiet {
        fn wake(&self) {}
    }

    /// Records its own drop in the shared event log.
    struct Probe(Rc<RefCell<Vec<&'static str>>>);

    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.borrow_mut().push("dropped");
        }
    }

    #[test]
    fn disposing_an_owner_releases_its_context_cleanup_error_handler_object_and_const() {
        let rt = Runtime::with_id(AppId(1), Rc::new(Quiet), 0);
        let at = Location::caller();
        let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let owner = rt.create_owner(None, at);

        rt.provide_context(owner, Rc::new(Probe(log.clone())));
        let cleanup_log = log.clone();
        rt.add_cleanup(
            owner,
            Box::new(move || cleanup_log.borrow_mut().push("cleanup")),
        );
        let handler_probe = Probe(log.clone());
        rt.set_error_handler(
            owner,
            Rc::new(move |_: UiError| {
                let _ = &handler_probe;
            }),
        );
        let object_log = log.clone();
        rt.insert_object(
            owner,
            Rc::new(Probe(log.clone())),
            "test object",
            Some(Box::new(move |_: &Runtime| {
                object_log.borrow_mut().push("object");
            })),
            at,
        );
        rt.create_const(owner, Probe(log.clone()), at);

        assert!(rt.owner_alive(owner));
        assert!(log.borrow().is_empty());

        rt.dispose_owner(owner);

        assert!(!rt.owner_alive(owner));
        let events = log.borrow();
        let count = |name: &str| events.iter().filter(|event| **event == name).count();
        assert_eq!(count("cleanup"), 1, "the cleanup runs once");
        assert_eq!(count("object"), 1, "the object's dispose hook runs once");
        assert_eq!(
            count("dropped"),
            4,
            "context, handler, object and const are all dropped"
        );
    }
}
