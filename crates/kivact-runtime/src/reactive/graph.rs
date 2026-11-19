//! Role: Reactive graph with three-color push-pull propagation.
//!
//! Reactive graph: sources, memos and effects with three-colour push-pull
//! propagation (Clean/Check/Dirty). Writes colour subscribers eagerly and
//! queue effects; memos are pulled lazily and cut off on `PartialEq`, so no
//! reader can observe a glitch. Edges are stored on both ends with
//! back-indices, so unlinking one edge is O(1) (no O(N^2) teardown).
//!
//! Invariants:
//! - Every write goes through `Runtime::write_node`: it marks the direct
//!   subscribers Dirty and the transitive memos Check, and queues the
//!   effects. A memo checks its dependencies when read; a recomputed value
//!   equal to the previous one (`PartialEq`) does not raise its revision
//!   (cutoff), so reading a memo always yields the current value.
//! - Effects run in creation order (`turn`): the effect of a parent flow
//!   runs before those of its children, so a child it disposes never runs.
//! - An effect that writes, while running, a signal it read is detected by
//!   the write epoch and the source's `changed_at`, and is queued again.
//! - A memo read while its own `with` borrow is active, after a dependency
//!   changed, cannot be replaced: that is a located `BorrowedWrite`. A memo
//!   that reads itself is a `Cycle`.
//! - Closures that only compute a value (flow conditions, list items and
//!   keys, the source of `dynamic` and of a resource, the target of an
//!   animated value, ...) run pure (`Runtime::pure_section`): a write or a
//!   side effect in them (cancel, focus, a platform request, ...) is
//!   rejected before it changes any state.

use std::any::Any;
use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use super::arena::Tomb;
use crate::reactive::NodeId;
use crate::reactive::arena::Key;
use crate::reactive::data::{CompState, Compute, EffectCell, MemoCell, RNode, Runtime, Staleness};
use crate::reactive::data::{
    EffectKind, EffectNode, MemoNode, NodeData, NodeInfo, NodeMeta, SourceCell, SourceNode,
};
use crate::reactive::error::{ErrorKind, misuse_at, new_error};
use crate::reactive::links::Links;

impl<T: 'static> Compute for MemoCell<T> {
    fn into_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn run(&self) -> bool {
        let new = {
            let old = self.value.borrow();
            (self.f)(old.as_ref())
        };
        let mut slot = self.value.borrow_mut();
        let same = match (&*slot, self.eq) {
            (Some(old), Some(eq)) => eq(old, &new),
            _ => false,
        };
        if same {
            return false;
        }
        *slot = Some(new);
        drop(slot);
        true
    }

    fn busy(&self) -> bool {
        self.value.try_borrow_mut().is_err()
    }
}

impl Compute for EffectCell {
    fn into_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn run(&self) -> bool {
        (self.f)();
        true
    }
}

#[derive(Clone, Copy)]
enum WatchPhase {
    Initial,
    Watching,
}

struct WatcherCell<T> {
    source: Box<dyn Fn() -> T>,
    callback: Box<dyn Fn(&T)>,
    phase: std::cell::Cell<WatchPhase>,
    app: crate::reactive::AppId,
}
impl<T: 'static> Compute for WatcherCell<T> {
    fn into_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn run(&self) -> bool {
        let Some(rt) = crate::reactive::registry::try_runtime(self.app) else {
            return false;
        };
        let initial = matches!(
            self.phase.replace(WatchPhase::Watching),
            WatchPhase::Initial
        );
        let value = {
            let _pure = rt.pure_section();
            (self.source)()
        };
        if !initial {
            let _untracked = rt.untracked_frame();
            (self.callback)(&value);
        }
        true
    }
}

/// Register a source watcher in the current setup owner: `callback` runs on
/// every later change of `source`, not for its first value.
#[track_caller]
pub(crate) fn watch<T: 'static>(
    source: impl Fn() -> T + 'static,
    kind: EffectKind,
    callback: impl Fn(&T) + 'static,
) {
    let at = Location::caller();
    let (rt, owner) = crate::reactive::scope::setup_scope("watch", at);
    rt.watch(owner, kind, source, callback, at);
}

impl Runtime {
    /// Whether new graph nodes keep their type and creation site
    /// (`Diag::node_meta`): always with the `devtools` feature, else while
    /// a recorder is on (12 章 ST-06).
    fn keeps_node_meta(&self) -> bool {
        cfg!(feature = "devtools") || self.diag.borrow().record.any()
    }

    /// Where node `key` was created, if its diagnostics are kept.
    pub(crate) fn node_at(&self, key: Key) -> Option<&'static Location<'static>> {
        self.diag.borrow().node_meta.get(&key).map(|m| m.at)
    }

    fn insert_node(
        &self,
        owner: Key,
        data: NodeData,
        at: &'static Location<'static>,
        ty: &'static str,
    ) -> Key {
        let state = if matches!(data, NodeData::Source(_)) {
            CompState::Clean
        } else {
            CompState::Stale(Staleness::Dirty)
        };
        let key = self.graph.borrow_mut().insert(RNode {
            owner,
            owner_index: 0,
            state,
            data,
        });
        if self.keeps_node_meta() {
            self.diag
                .borrow_mut()
                .node_meta
                .insert(key, NodeMeta { at, ty });
        }
        let index = self.owners.borrow_mut().get_mut(owner).map(|o| {
            o.nodes.push(key);
            o.nodes.len() as u32 - 1
        });
        match index {
            Some(i) => {
                if let Some(n) = self.graph.borrow_mut().get_mut(key) {
                    n.owner_index = i;
                }
            }
            // A gone owner: released at once (the key is a tombstone).
            None => self.release_orphan_node(key, owner),
        }
        key
    }

    /// Create a source node of `owner` holding `value`.
    pub(crate) fn create_source<T: 'static>(
        &self,
        owner: Key,
        value: T,
        at: &'static Location<'static>,
    ) -> Key {
        let cell: Rc<dyn Any> = Rc::new(SourceCell {
            value: RefCell::new(value),
        });
        let ty = std::any::type_name::<T>();
        let data = NodeData::Source(SourceNode {
            cell,
            subs: Links::default(),
            changed_at: kivact_types::cell::Generation::new(),
        });
        self.insert_node(owner, data, at, ty)
    }

    pub(crate) fn create_memo<T: 'static>(
        &self,
        owner: Key,
        f: Box<dyn Fn(Option<&T>) -> T>,
        eq: Option<fn(&T, &T) -> bool>,
        at: &'static Location<'static>,
    ) -> Key {
        let cell = Rc::new(MemoCell {
            value: RefCell::new(None),
            f,
            eq,
        });
        let ty = std::any::type_name::<T>();
        let data = NodeData::Memo(MemoNode {
            comp: cell,
            subs: Links::default(),
            deps: Links::default(),
        });
        self.insert_node(owner, data, at, ty)
    }

    /// Create an effect. Render/Internal effects run once immediately (their
    /// first output is part of the mount); User effects are queued.
    pub(crate) fn create_effect(
        &self,
        owner: Key,
        kind: EffectKind,
        f: Box<dyn Fn()>,
        at: &'static Location<'static>,
    ) -> Key {
        let cell = Rc::new(EffectCell { f });
        let seq = self.next_seq();
        let data = NodeData::Effect(EffectNode {
            comp: cell,
            deps: Links::default(),
            kind,
            pass_runs: 0,
            seq,
            cleanups: None,
        });
        let key = self.insert_node(owner, data, at, "effect");
        if !self.node_alive(key) {
            return key;
        }
        match kind {
            EffectKind::User => {
                if self.queue_effect(key) {
                    self.request_turn();
                }
            }
            _ => self.run_effect(key),
        }
        key
    }

    pub(crate) fn watch<T: 'static>(
        &self,
        owner: Key,
        kind: EffectKind,
        source: impl Fn() -> T + 'static,
        callback: impl Fn(&T) + 'static,
        at: &'static Location<'static>,
    ) -> Key {
        let cell = Rc::new(WatcherCell {
            source: Box::new(source),
            callback: Box::new(callback),
            phase: std::cell::Cell::new(WatchPhase::Initial),
            app: self.id,
        });
        let data = NodeData::Watcher(EffectNode {
            comp: cell,
            deps: Links::default(),
            kind,
            pass_runs: 0,
            seq: self.next_seq(),
            cleanups: None,
        });
        let key = self.insert_node(owner, data, at, "watcher");
        if !self.node_alive(key) {
            return key;
        }
        // Like `create_effect`: a Render or Internal watcher subscribes now,
        // so a change before the next stabilization reaches its callback.
        match kind {
            EffectKind::User => {
                if self.queue_effect(key) {
                    self.request_turn();
                }
            }
            _ => self.run_effect(key),
        }
        key
    }

    fn queue_effect(&self, key: Key) -> bool {
        let mut graph = self.graph.borrow_mut();
        let Some(n) = graph.get_mut(key) else {
            return false;
        };
        let (NodeData::Effect(e) | NodeData::Watcher(e)) = &n.data else {
            return false;
        };
        let (kind, seq) = (e.kind, e.seq);
        self.sched.borrow_mut().push(&mut n.state, kind, seq, key)
    }
    fn dequeue_effect(&self, key: Key) {
        let mut graph = self.graph.borrow_mut();
        let Some(n) = graph.get_mut(key) else {
            return;
        };
        if let NodeData::Effect(e) | NodeData::Watcher(e) = &n.data {
            let (kind, seq) = (e.kind, e.seq);
            self.sched.borrow_mut().remove(&mut n.state, kind, seq, key);
        }
    }

    /// Whether graph node `key` exists.
    pub(crate) fn node_alive(&self, key: Key) -> bool {
        self.graph.borrow().contains(key)
    }

    fn disposed_panic(&self, key: Key, what: &str, at: &'static Location<'static>) -> ! {
        let tomb = self.graph.borrow().tomb(key);
        misuse_at(
            ErrorKind::Disposed {
                created_at: tomb.and_then(|t| t.created_at),
                disposed_by: tomb.and_then(|t| t.disposed_by),
            },
            format!("{what} read after its owner was disposed"),
            at,
        )
    }

    /// Bring a node up to date, register the read if `track`, and return its cell.
    pub(crate) fn read_node(
        &self,
        key: Key,
        track: bool,
        at: &'static Location<'static>,
    ) -> Option<Rc<dyn Any>> {
        let (cell, memo) = {
            let g = self.graph.borrow();
            let n = g.get(key)?;
            let memo = matches!(n.data, NodeData::Memo(_)).then_some(n.state);
            (n.cell(), memo)
        };
        if let Some(state) = memo {
            if state == CompState::Running {
                let path = self.frame_path();
                misuse_at(
                    ErrorKind::Cycle { path },
                    "memo reads itself (dependency cycle)",
                    at,
                );
            }
            if state != CompState::Clean {
                self.refresh(key);
            }
        }
        if track {
            self.track(key);
        }
        Some(cell)
    }

    pub(crate) fn read_or_panic(
        &self,
        key: Key,
        track: bool,
        what: &str,
        at: &'static Location<'static>,
    ) -> Rc<dyn Any> {
        match self.read_node(key, track, at) {
            Some(c) => c,
            None => self.disposed_panic(key, what, at),
        }
    }

    /// A write to a removed signal: ignored, reported once as a located
    /// `WriteAfterDispose` diagnostic.
    pub(crate) fn write_after_dispose(&self, at: &'static Location<'static>) {
        let err = new_error(
            ErrorKind::WriteAfterDispose,
            "write to a disposed signal ignored",
            Some(at),
        );
        self.diag.borrow_mut().record_diagnostic(err);
        // Close a turn record so the note is visible in `trace()`.
        self.request_turn();
    }

    /// The single write entry: purity check, immediate value change by
    /// `mutate`, deferred notification. Returns whether the node is alive.
    pub(crate) fn write_node(
        &self,
        key: Key,
        at: &'static Location<'static>,
        mutate: impl FnOnce(&Rc<dyn Any>) -> bool,
    ) -> bool {
        if self.is_pure() {
            misuse_at(
                ErrorKind::WriteInPureContext,
                "signal written inside a memo or render effect",
                at,
            );
        }
        let cell = self.graph.borrow().get(key).and_then(|n| match &n.data {
            NodeData::Source(s) => Some(Rc::clone(&s.cell)),
            _ => None,
        });
        let Some(cell) = cell else {
            self.write_after_dispose(at);
            return false;
        };
        if mutate(&cell) {
            let turn = self.diag.borrow().turn();
            let mut diag = self.diag.borrow_mut();
            if diag.record.any() {
                diag.node_written.insert(key, (at, turn));
            }
            let meta = diag.node_meta.get(&key).copied();
            if let Some(meta) = meta {
                diag.record_write(meta.ty, meta.at, at);
                diag.detail.write(NodeId(key), meta.ty, meta.at, at, true);
            }
            drop(diag);
            self.notify_subscribers(key);
        } else {
            let mut diag = self.diag.borrow_mut();
            if diag.detail.is_recording() {
                let written = diag.node_meta.get(&key).map(|m| (m.ty, m.at));
                if let Some((ty, created_at)) = written {
                    diag.detail.write(NodeId(key), ty, created_at, at, false);
                }
            }
        }
        true
    }

    /// Colour subscribers of a changed source (direct: Dirty, transitive: Check)
    /// and queue affected effects.
    pub(crate) fn notify_subscribers(&self, src: Key) {
        let mut stack = std::mem::take(&mut *self.scratch.borrow_mut());
        stack.clear();
        let mut queued_any = false;
        let mut diag = self.diag.borrow_mut();
        let mut write_epoch = self.write_epoch.get();
        let epoch = write_epoch.next();
        self.write_epoch.set(write_epoch);
        {
            let mut g = self.graph.borrow_mut();
            let mut sched = self.sched.borrow_mut();
            if let Some(n) = g.get_mut(src) {
                if let NodeData::Source(s) = &mut n.data {
                    s.changed_at = epoch;
                }
                for (s, _) in n.subs() {
                    diag.detail.edge(NodeId(src), NodeId(*s));
                }
                stack.extend(n.subs().iter().map(|(s, _)| (*s, Staleness::Dirty)));
            }
            while let Some((k, color)) = stack.pop() {
                let Some(n) = g.get_mut(k) else { continue };
                // A source never reads anything, so it is never a subscriber.
                let effect = match &n.data {
                    NodeData::Source(_) => continue,
                    NodeData::Memo(_) => None,
                    NodeData::Effect(e) | NodeData::Watcher(e) => Some((e.kind, e.seq)),
                };
                let was_clean = n.state == CompState::Clean;
                if !n.state.mark(color) {
                    continue;
                }
                match effect {
                    None => {
                        if was_clean {
                            for (s, _) in n.subs() {
                                diag.detail.edge(NodeId(k), NodeId(*s));
                            }
                            stack.extend(n.subs().iter().map(|(s, _)| (*s, Staleness::Check)));
                        }
                    }
                    Some((kind, seq)) => {
                        queued_any |= sched.push(&mut n.state, kind, seq, k);
                    }
                }
            }
        }
        drop(diag);
        *self.scratch.borrow_mut() = stack;
        if queued_any {
            self.request_turn();
        }
    }

    /// Pull a memo or effect up to date (reactively's update-if-necessary).
    pub(crate) fn refresh(&self, key: Key) {
        let state = match self
            .graph
            .borrow()
            .get(key)
            .and_then(|n| n.is_computation().then_some(n.state))
        {
            Some(s) => s,
            None => return,
        };
        if state.staleness() == Some(Staleness::Check) {
            let mut i = 0;
            loop {
                let dep = {
                    let g = self.graph.borrow();
                    let Some(n) = g.get(key).filter(|n| n.is_computation()) else {
                        return;
                    };
                    if n.state.staleness() == Some(Staleness::Dirty) {
                        break;
                    }
                    match n.deps().get(i) {
                        Some((d, _)) => *d,
                        None => break,
                    }
                };
                let needs = {
                    let g = self.graph.borrow();
                    g.get(dep).is_some_and(|d| {
                        matches!(&d.data, NodeData::Memo(_)) && d.state.staleness().is_some()
                    })
                };
                if needs {
                    self.refresh(dep);
                }
                i += 1;
            }
        }
        let dirty = self
            .graph
            .borrow()
            .get(key)
            .filter(|n| n.is_computation())
            .is_some_and(|n| n.state.staleness() == Some(Staleness::Dirty));
        if dirty {
            self.recompute(key);
        } else if let Some(n) = self
            .graph
            .borrow_mut()
            .get_mut(key)
            .filter(|n| n.is_computation())
        {
            n.state = CompState::Clean;
        }
    }

    /// Whether memo or effect `key` is out of date and not running now.
    pub(crate) fn needs_run(&self, key: Key) -> bool {
        self.graph
            .borrow()
            .get(key)
            .filter(|n| n.is_computation())
            .is_some_and(|n| n.state.staleness().is_some())
    }

    fn recompute(&self, key: Key) {
        // Pulling dependencies may have queued this effect again.
        self.dequeue_effect(key);
        // A memo whose value is borrowed by a running `with` cannot be
        // replaced: report a structured misuse and keep it dirty.
        let busy = self
            .graph
            .borrow()
            .get(key)
            .and_then(|n| n.comp().filter(|c| c.busy()).map(|_| ()));
        if busy.is_some() {
            crate::reactive::error::misuse(new_error(
                ErrorKind::BorrowedWrite,
                "memo re-read while its own `with` borrow is active after a dependency changed; read it after `with` returns",
                self.node_at(key),
            ));
        }
        let (compute, effect, owner) = {
            let mut g = self.graph.borrow_mut();
            let Some(n) = g.get_mut(key) else { return };
            let owner = n.owner;
            let effect = match &n.data {
                NodeData::Effect(e) | NodeData::Watcher(e) => Some(e.kind),
                _ => None,
            };
            // Only memos and effects are refreshed; a source has nothing to run.
            let Some(compute) = n.comp().cloned() else {
                return;
            };
            n.state = CompState::Running;
            (compute, effect, owner)
        };
        let label = match effect {
            None => "memo",
            Some(EffectKind::Render) => "render",
            Some(EffectKind::Internal) => "internal",
            Some(EffectKind::User) => "effect",
        };
        let at = self.node_at(key);
        if let Some(at) = at {
            self.diag.borrow_mut().record_run(label, at);
        }
        let (epoch_before, is_effect) = (self.write_epoch.get(), effect.is_some());
        if is_effect {
            self.run_node_cleanups(key, owner);
        }
        let pure = matches!(effect, None | Some(EffectKind::Render));
        let changed = {
            let writes_for_node = matches!(effect, Some(EffectKind::Render | EffectKind::Internal));
            let guard = self.tracking_frame(key, owner, pure, is_effect, writes_for_node);
            let changed = compute.run();
            let deps = guard.take_deps();
            drop(guard);
            self.set_deps(key, &deps);
            self.recycle_deps(deps);
            changed
        };
        if let Some(at) = at {
            self.diag
                .borrow_mut()
                .detail
                .run(NodeId(key), label, at, changed);
        }
        let is_memo_change = {
            let mut g = self.graph.borrow_mut();
            let Some(n) = g.get_mut(key) else { return };
            n.state = CompState::Clean;
            changed && matches!(n.data, NodeData::Memo(_))
        };
        if is_effect && self.write_epoch.get() != epoch_before {
            self.recheck_after_own_writes(key, epoch_before);
        }
        if is_memo_change {
            self.mark_memo_subscribers(key);
        }
    }

    /// Mark the subscribers of a memo whose value changed as dirty and queue
    /// the effects among them (no list is collected).
    fn mark_memo_subscribers(&self, key: Key) {
        let mut g = self.graph.borrow_mut();
        let mut sched = self.sched.borrow_mut();
        let count = g.get(key).map_or(0, |n| n.subs().len());
        for i in 0..count {
            let Some(s) = g.get(key).and_then(|n| n.subs().get(i)).map(|(s, _)| *s) else {
                break;
            };
            let Some(n) = g.get_mut(s) else { continue };
            if !n.is_computation() {
                continue;
            }
            let queue = match &n.data {
                NodeData::Effect(e) | NodeData::Watcher(e) => Some((e.kind, e.seq)),
                _ => None,
            };
            // A running subscriber is pulling this memo right now and reads
            // the new value; do not re-mark it.
            if n.state.mark(Staleness::Dirty)
                && let Some((kind, seq)) = queue
            {
                sched.push(&mut n.state, kind, seq, s);
            }
        }
    }

    /// An effect that wrote a signal it read during the same run saw a stale
    /// value: mark it dirty so it runs again (cycles are bounded by the turn).
    fn recheck_after_own_writes(&self, key: Key, epoch_before: kivact_types::cell::Generation) {
        let mut g = self.graph.borrow_mut();
        let stale = match g.get(key) {
            Some(n) => n
                .deps()
                .iter()
                .filter_map(|(d, _)| match g.get(*d).map(|n| &n.data) {
                    Some(NodeData::Source(s)) if s.changed_at > epoch_before => {
                        Some(Staleness::Dirty)
                    }
                    Some(NodeData::Memo(_))
                        if g.get(*d).is_some_and(|m| m.state.staleness().is_some()) =>
                    {
                        Some(Staleness::Check)
                    }
                    _ => None,
                })
                .max(),
            None => return,
        };
        let Some(stale) = stale else { return };
        let Some(n) = g.get_mut(key) else { return };
        let (NodeData::Effect(_) | NodeData::Watcher(_)) = &n.data else {
            return;
        };
        n.state.mark(stale);
        drop(g);
        if self.queue_effect(key) {
            self.request_turn();
        }
    }

    fn run_node_cleanups(&self, key: Key, owner: Key) {
        let cleanups = match self.graph.borrow_mut().get_mut(key).map(|n| &mut n.data) {
            Some(NodeData::Effect(e) | NodeData::Watcher(e)) => e.cleanups.take(),
            _ => return,
        };
        let Some(cleanups) = cleanups else { return };
        let _g = self.handler_frame(Some(owner));
        for c in cleanups.into_iter().rev() {
            c();
        }
    }

    /// Run an effect now if it is not clean (at creation, by the scheduler,
    /// and by a selector catching up).
    pub(crate) fn run_effect(&self, key: Key) {
        self.dequeue_effect(key);
        self.refresh(key);
    }

    /// Replace the dependency edges of `node` with `new` (read order).
    fn set_deps(&self, node: Key, new: &[Key]) {
        let mut g = self.graph.borrow_mut();
        let same = match g.get(node) {
            Some(n) => {
                n.deps().len() == new.len() && n.deps().iter().zip(new).all(|((a, _), b)| a == b)
            }
            None => return,
        };
        if same {
            return;
        }
        let emptied = unlink_deps(&mut g, node);
        for &src in new {
            let dep_index = match g.get(node) {
                Some(n) => n.deps().len() as u32,
                None => return,
            };
            // Only sources and memos are read.
            let Some(subs) = g.get_mut(src).and_then(RNode::subs_mut) else {
                continue;
            };
            subs.push((node, dep_index));
            let sub_index = subs.len() as u32 - 1;
            if let Some(deps) = g.get_mut(node).and_then(RNode::deps_mut) {
                deps.push((src, sub_index));
            }
        }
        drop(g);
        self.run_unsubscribed(emptied);
    }

    /// Run `hook` once when source `key` loses its last subscriber.
    pub(crate) fn on_unsubscribed(&self, key: Key, hook: Box<dyn FnOnce(&Runtime)>) {
        self.unsubscribed.borrow_mut().insert(key, hook);
    }

    /// Run the hooks of the sources in `emptied` that still have no
    /// subscriber (no graph borrow is held).
    fn run_unsubscribed(&self, emptied: Vec<Key>) {
        for key in emptied {
            let unread = self
                .graph
                .borrow()
                .get(key)
                .is_some_and(|n| n.subs().is_empty());
            if !unread {
                continue;
            }
            let hook = self.unsubscribed.borrow_mut().remove(&key);
            if let Some(hook) = hook {
                hook(self);
            }
        }
    }

    /// Remove a node from the graph (owner disposal). Returns its cell so the
    /// caller can drop user values outside of any borrow.
    pub(crate) fn remove_node(
        &self,
        key: Key,
        disposed_by: Option<&'static Location<'static>>,
    ) -> Option<RNode> {
        self.dequeue_effect(key);
        let mut g = self.graph.borrow_mut();
        let emptied = unlink_deps(&mut g, key);
        // Unlink subscribers: remove this node from each subscriber's deps.
        let subs = g
            .get_mut(key)?
            .subs_mut()
            .map(std::mem::take)
            .unwrap_or_default();
        for (sub, dep_index) in subs {
            remove_dep_entry(&mut g, sub, dep_index as usize);
        }
        let created_at = self.diag.borrow_mut().node_meta.remove(&key).map(|m| m.at);
        self.diag.borrow_mut().node_written.remove(&key);
        let removed = g.remove(
            key,
            Some(Tomb {
                created_at,
                disposed_by,
            }),
        );
        drop(g);
        self.unsubscribed.borrow_mut().remove(&key);
        self.run_unsubscribed(emptied);
        removed
    }

    /// Remove a single owned node before its owner is disposed.
    pub(crate) fn remove_owned_node(&self, key: Key) -> Option<RNode> {
        let (owner, index) = self
            .graph
            .borrow()
            .get(key)
            .map(|n| (n.owner, n.owner_index as usize))?;
        let moved = self.owners.borrow_mut().get_mut(owner).and_then(|o| {
            if index < o.nodes.len() && o.nodes[index] == key {
                o.nodes.swap_remove(index);
                o.nodes.get(index).copied()
            } else {
                None
            }
        });
        if let Some(m) = moved
            && let Some(n) = self.graph.borrow_mut().get_mut(m)
        {
            n.owner_index = index as u32;
        }
        self.remove_node(key, None)
    }
}

/// Remove all dependency edges of `node` (both directions).
fn unlink_deps(g: &mut crate::reactive::arena::Arena<RNode>, node: Key) -> Vec<Key> {
    let mut emptied = Vec::new();
    let Some(node_deps) = g.get_mut(node).and_then(RNode::deps_mut) else {
        return emptied;
    };
    let deps = std::mem::take(node_deps);
    for (src, sub_index) in deps {
        let moved = match g.get_mut(src).and_then(RNode::subs_mut) {
            Some(subs) => {
                let i = sub_index as usize;
                if i >= subs.len() {
                    continue;
                }
                subs.swap_remove(i);
                if subs.is_empty() {
                    emptied.push(src);
                }
                subs.get(i).copied().map(|(m, k)| (m, k, i))
            }
            None => None,
        };
        if let Some((m, k, i)) = moved
            && let Some(e) = g
                .get_mut(m)
                .and_then(RNode::deps_mut)
                .and_then(|d| d.get_mut(k as usize))
        {
            e.1 = i as u32;
        }
    }
    emptied
}

/// Remove `sub.deps[index]` (whose source is being disposed) keeping read order.
fn remove_dep_entry(g: &mut crate::reactive::arena::Arena<RNode>, sub: Key, index: usize) {
    let fix: Vec<(Key, u32, u32)> = {
        let Some(deps) = g.get_mut(sub).and_then(RNode::deps_mut) else {
            return;
        };
        if index >= deps.len() {
            return;
        }
        deps.remove(index);
        // Fix back-indices of the following deps in their sources.
        deps[index..]
            .iter()
            .enumerate()
            .map(|(j, (src, si))| (*src, *si, (index + j) as u32))
            .collect()
    };
    for (src, si, new_dep_index) in fix {
        if let Some(e) = g
            .get_mut(src)
            .and_then(RNode::subs_mut)
            .and_then(|s| s.get_mut(si as usize))
        {
            e.1 = new_dep_index;
        }
    }
}

/// The memo cell of `T` behind `cell`, if it is one (`read_node` computed
/// its value first; a memo that reads itself is a `Cycle` misuse there).
pub(crate) fn memo_cell<T: 'static>(cell: &Rc<dyn Any>) -> Option<&MemoCell<T>> {
    cell.downcast_ref::<MemoCell<T>>()
}

/// Read helpers for typed handles.
pub(crate) fn source_cell<T: 'static>(cell: &Rc<dyn Any>) -> Option<&SourceCell<T>> {
    cell.downcast_ref::<SourceCell<T>>()
}

impl Runtime {
    /// Typed value cell of a source node (internal fast paths).
    pub(crate) fn source_rc<T: 'static>(&self, key: Key) -> Option<Rc<SourceCell<T>>> {
        let cell = self.graph.borrow().get(key).map(|n| n.cell())?;
        cell.downcast::<SourceCell<T>>().ok()
    }

    /// Snapshot of node `key`, or `None` if it no longer exists.
    pub(crate) fn node_snapshot(&self, key: Key) -> Option<NodeInfo> {
        let (meta, written) = {
            let diag = self.diag.borrow();
            (
                diag.node_meta.get(&key).copied()?,
                diag.node_written.get(&key).copied(),
            )
        };
        let g = self.graph.borrow();
        let n = g.get(key)?;
        Some(NodeInfo {
            kind: n.kind(),
            ty: meta.ty,
            at: meta.at,
            written,
            deps: n.deps().iter().map(|(d, _)| NodeId(*d)).collect(),
        })
    }
}

#[cfg(test)]
mod meta_tests {
    //! Role: Node diagnostics are kept with the devtools feature from the start, and without it only while recording.

    use std::panic::Location;
    use std::rc::Rc;

    use crate::reactive::NodeId;
    use crate::reactive::data::Runtime;
    use crate::reactive::diag::RecordBits;
    use crate::reactive::host_types::Host;

    struct Quiet;
    impl Host for Quiet {
        fn wake(&self) {}
    }

    #[test]
    fn node_meta_follows_the_devtools_feature() {
        let rt = Runtime::register(
            Rc::new(Quiet),
            0,
            crate::TaskState::new(None, crate::ThreadWaker::new(|| {})),
        );
        let at = Location::caller();
        // The root owner takes the first owner slot, as `Runtime::new` makes it.
        assert_eq!(rt.create_owner(None, at), rt.root);
        let key = rt.create_source(rt.root, 1u32, at);
        // Recording starts after the node was created.
        rt.set_recorder(RecordBits::DEVTOOLS, true);
        let info = rt.node_info(NodeId(key));
        assert_eq!(
            info.is_some(),
            cfg!(feature = "devtools"),
            "type and site are kept exactly with the feature"
        );
        if let Some(info) = info {
            assert_eq!(info.ty, "u32");
        }
        rt.shutdown();
    }
}
