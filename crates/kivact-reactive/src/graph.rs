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
use crate::NodeId;
use crate::arena::Key;
use crate::data::{Color, Compute, EffectCell, MemoCell, RNode, Runtime};
use crate::data::{EffectKind, Kind, NodeInfo, SourceCell};
use crate::error::{ErrorKind, misuse_at, new_error};

impl<T: 'static> Compute for MemoCell<T> {
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
    fn run(&self) -> bool {
        (self.f)();
        true
    }
}

impl Runtime {
    fn insert_node(
        &self,
        owner: Key,
        kind: Kind,
        cell: Rc<dyn Any>,
        compute: Option<Rc<dyn Compute>>,
        at: &'static Location<'static>,
        ty: &'static str,
    ) -> Key {
        let state = if matches!(kind, Kind::Source) {
            Color::Clean
        } else {
            Color::Dirty
        };
        let seq = self.next_seq();
        let key = self.graph.borrow_mut().insert(RNode {
            kind,
            owner,
            owner_index: 0,
            cell,
            compute,
            deps: Vec::new(),
            subs: Vec::new(),
            state,
            seq,
            at,
            queued: false,
            running: false,
            cleanups: Vec::new(),
            changed_at: 0,
            ty,
            written: None,
            stab_epoch: 0,
            stab_runs: 0,
        });
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
        self.insert_node(owner, Kind::Source, cell, None, at, ty)
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
        let compute: Rc<dyn Compute> = cell.clone();
        let ty = std::any::type_name::<T>();
        self.insert_node(owner, Kind::Memo, cell, Some(compute), at, ty)
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
        let compute: Rc<dyn Compute> = cell.clone();
        let key = self.insert_node(owner, Kind::Effect(kind), cell, Some(compute), at, "effect");
        if !self.node_alive(key) {
            return key;
        }
        match kind {
            EffectKind::User => {
                let seq = self.graph.borrow().get(key).map(|n| n.seq).unwrap_or(0);
                if let Some(n) = self.graph.borrow_mut().get_mut(key) {
                    n.queued = true;
                }
                self.sched.borrow_mut().push(kind, seq, key);
                self.request_turn();
            }
            _ => self.run_effect(key),
        }
        key
    }

    /// Whether graph node `key` exists.
    pub(crate) fn node_alive(&self, key: Key) -> bool {
        self.graph.borrow().contains(key)
    }

    fn disposed_panic(&self, key: Key, what: &str, at: &'static Location<'static>) -> ! {
        let tomb = self.graph.borrow().tomb(key);
        misuse_at(
            ErrorKind::Disposed {
                created_at: tomb.map(|t| t.created_at),
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
        let (cell, kind, state, running) = {
            let g = self.graph.borrow();
            let n = g.get(key)?;
            (n.cell.clone(), n.kind, n.state, n.running)
        };
        if kind == Kind::Memo {
            if running {
                let path = self.frame_path();
                misuse_at(
                    ErrorKind::Cycle { path },
                    "memo reads itself (dependency cycle)",
                    at,
                );
            }
            if state != Color::Clean {
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

    /// The single write entry: purity check, immediate value change by
    /// `mutate`, deferred notification.
    pub(crate) fn write_node(
        &self,
        key: Key,
        at: &'static Location<'static>,
        mutate: impl FnOnce(&Rc<dyn Any>) -> bool,
    ) {
        if self.is_pure() {
            misuse_at(
                ErrorKind::WriteInPureContext,
                "signal written inside a memo or render effect",
                at,
            );
        }
        let cell = self.graph.borrow().get(key).map(|n| n.cell.clone());
        let Some(cell) = cell else {
            let err = new_error(
                ErrorKind::WriteAfterDispose,
                "write to a disposed signal ignored",
                Some(at),
            );
            self.diag.borrow_mut().record_diagnostic(err);
            // Close a turn record so the note is visible in `trace()`.
            self.request_turn();
            return;
        };
        if mutate(&cell) {
            let turn = self.diag.borrow().turn();
            let written = self.graph.borrow_mut().get_mut(key).map(|n| {
                n.written = Some((at, turn));
                (n.ty, n.at)
            });
            if let Some((ty, created_at)) = written {
                self.diag.borrow_mut().record_write(ty, created_at, at);
            }
            self.notify_subscribers(key);
        }
    }

    /// Colour subscribers of a changed source (direct: Dirty, transitive: Check)
    /// and queue affected effects.
    pub(crate) fn notify_subscribers(&self, src: Key) {
        let mut stack = std::mem::take(&mut *self.scratch.borrow_mut());
        stack.clear();
        let mut queued_any = false;
        let epoch = self.write_epoch.get() + 1;
        self.write_epoch.set(epoch);
        {
            let mut g = self.graph.borrow_mut();
            let mut sched = self.sched.borrow_mut();
            if let Some(n) = g.get_mut(src) {
                n.changed_at = epoch;
                stack.extend(n.subs.iter().map(|(s, _)| (*s, Color::Dirty)));
            }
            while let Some((k, color)) = stack.pop() {
                let Some(n) = g.get_mut(k) else { continue };
                if n.state >= color {
                    continue;
                }
                let was_clean = n.state == Color::Clean;
                n.state = color;
                match n.kind {
                    Kind::Effect(kind) => {
                        if !n.queued {
                            n.queued = true;
                            sched.push(kind, n.seq, k);
                            queued_any = true;
                        }
                    }
                    Kind::Memo if was_clean => {
                        stack.extend(n.subs.iter().map(|(s, _)| (*s, Color::Check)));
                    }
                    _ => {}
                }
            }
        }
        *self.scratch.borrow_mut() = stack;
        if queued_any {
            self.request_turn();
        }
    }

    /// Pull a memo or effect up to date (reactively's update-if-necessary).
    pub(crate) fn refresh(&self, key: Key) {
        let state = match self.graph.borrow().get(key) {
            Some(n) => n.state,
            None => return,
        };
        if state == Color::Check {
            let mut i = 0;
            loop {
                let dep = {
                    let g = self.graph.borrow();
                    let Some(n) = g.get(key) else { return };
                    if n.state == Color::Dirty {
                        break;
                    }
                    match n.deps.get(i) {
                        Some((d, _)) => *d,
                        None => break,
                    }
                };
                let needs = {
                    let g = self.graph.borrow();
                    g.get(dep).is_some_and(|d| {
                        d.kind == Kind::Memo && d.state != Color::Clean && !d.running
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
            .is_some_and(|n| n.state == Color::Dirty);
        if dirty {
            self.recompute(key);
        } else if let Some(n) = self.graph.borrow_mut().get_mut(key) {
            n.state = Color::Clean;
        }
    }

    fn recompute(&self, key: Key) {
        // A memo whose value is borrowed by a running `with` cannot be
        // replaced: report a structured misuse and keep it dirty.
        let busy = self
            .graph
            .borrow()
            .get(key)
            .and_then(|n| n.compute.as_ref().filter(|c| c.busy()).map(|_| n.at));
        if let Some(at) = busy {
            misuse_at(
                ErrorKind::BorrowedWrite,
                "memo re-read while its own `with` borrow is active after a dependency changed; read it after `with` returns",
                at,
            );
        }
        let (compute, kind, owner, at) = {
            let mut g = self.graph.borrow_mut();
            let Some(n) = g.get_mut(key) else { return };
            n.running = true;
            // Clean before running: a write during the run re-marks it.
            n.state = Color::Clean;
            (n.compute.clone(), n.kind, n.owner, n.at)
        };
        self.diag.borrow_mut().record_run(kind, at);
        // Props set by this run are attributed to it (diagnostics::why).
        let writer = matches!(
            kind,
            Kind::Effect(EffectKind::Render | EffectKind::Internal)
        )
        .then(|| self.writer.replace(Some(key)));
        let epoch_before = self.write_epoch.get();
        let Some(compute) = compute else { return };
        let is_effect = matches!(kind, Kind::Effect(_));
        if is_effect {
            self.run_node_cleanups(key, owner);
        }
        let pure = matches!(kind, Kind::Memo | Kind::Effect(EffectKind::Render));
        let changed = {
            let guard = self.tracking_frame(key, owner, pure, is_effect);
            let changed = compute.run();
            let deps = guard.take_deps();
            drop(guard);
            self.set_deps(key, deps);
            changed
        };
        if let Some(prev) = writer {
            self.writer.set(prev);
        }
        let subs = {
            let mut g = self.graph.borrow_mut();
            let Some(n) = g.get_mut(key) else { return };
            n.running = false;
            if kind == Kind::Memo && changed {
                n.subs.iter().map(|(s, _)| *s).collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };
        if is_effect && self.write_epoch.get() != epoch_before {
            self.recheck_after_own_writes(key, epoch_before);
        }
        if !subs.is_empty() {
            let mut g = self.graph.borrow_mut();
            let mut sched = self.sched.borrow_mut();
            for s in subs {
                // A running subscriber is pulling this memo right now and
                // reads the new value; do not re-mark it.
                if let Some(n) = g.get_mut(s)
                    && n.state != Color::Dirty
                    && !n.running
                {
                    n.state = Color::Dirty;
                    if let Kind::Effect(k) = n.kind
                        && !n.queued
                    {
                        n.queued = true;
                        sched.push(k, n.seq, s);
                    }
                }
            }
        }
    }

    /// An effect that wrote a signal it read during the same run saw a stale
    /// value: mark it dirty so it runs again (cycles are bounded by the turn).
    fn recheck_after_own_writes(&self, key: Key, epoch_before: u64) {
        let mut g = self.graph.borrow_mut();
        let stale = match g.get(key) {
            Some(n) => n
                .deps
                .iter()
                .any(|(d, _)| g.get(*d).is_some_and(|d| d.changed_at > epoch_before)),
            None => return,
        };
        if !stale {
            return;
        }
        let Some(n) = g.get_mut(key) else { return };
        n.state = Color::Dirty;
        if let Kind::Effect(k) = n.kind
            && !n.queued
        {
            n.queued = true;
            let seq = n.seq;
            drop(g);
            self.sched.borrow_mut().push(k, seq, key);
            self.request_turn();
        }
    }

    fn run_node_cleanups(&self, key: Key, owner: Key) {
        let cleanups = match self.graph.borrow_mut().get_mut(key) {
            Some(n) => std::mem::take(&mut n.cleanups),
            None => return,
        };
        if cleanups.is_empty() {
            return;
        }
        let _g = self.handler_frame(Some(owner));
        for c in cleanups.into_iter().rev() {
            c();
        }
    }

    /// Run an effect now if it is not clean (used at creation and by the scheduler).
    pub(crate) fn run_effect(&self, key: Key) {
        if let Some(n) = self.graph.borrow_mut().get_mut(key) {
            n.queued = false;
        } else {
            return;
        }
        self.refresh(key);
    }

    /// Replace the dependency edges of `node` with `new` (read order).
    fn set_deps(&self, node: Key, new: Vec<Key>) {
        let mut g = self.graph.borrow_mut();
        let same = match g.get(node) {
            Some(n) => {
                n.deps.len() == new.len() && n.deps.iter().zip(&new).all(|((a, _), b)| a == b)
            }
            None => return,
        };
        if same {
            return;
        }
        unlink_deps(&mut g, node);
        for src in new {
            let dep_index = match g.get(node) {
                Some(n) => n.deps.len() as u32,
                None => return,
            };
            let Some(s) = g.get_mut(src) else { continue };
            s.subs.push((node, dep_index));
            let sub_index = s.subs.len() as u32 - 1;
            if let Some(n) = g.get_mut(node) {
                n.deps.push((src, sub_index));
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
        let mut g = self.graph.borrow_mut();
        unlink_deps(&mut g, key);
        // Unlink subscribers: remove this node from each subscriber's deps.
        let subs = {
            let n = g.get_mut(key)?;
            std::mem::take(&mut n.subs)
        };
        for (sub, dep_index) in subs {
            remove_dep_entry(&mut g, sub, dep_index as usize);
        }
        let at = g.get(key).map(|n| n.at)?;
        g.remove(
            key,
            Some(Tomb {
                created_at: at,
                disposed_by,
            }),
        )
    }

    /// Current value of a memo, computing it if needed; a memo that has no
    /// value yet (it reads itself) is a `Cycle` misuse.
    pub(crate) fn memo_value<T: 'static, R>(
        &self,
        cell: &Rc<dyn Any>,
        f: impl FnOnce(&T) -> R,
        at: &'static Location<'static>,
    ) -> R {
        let Some(memo) = cell.downcast_ref::<MemoCell<T>>() else {
            misuse_at(ErrorKind::WrongApp, "handle type mismatch", at)
        };
        let v = memo.value.borrow();
        match v.as_ref() {
            Some(v) => f(v),
            None => {
                drop(v);
                misuse_at(
                    ErrorKind::Cycle {
                        path: self.frame_path(),
                    },
                    "memo value unavailable (cycle)",
                    at,
                )
            }
        }
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
fn unlink_deps(g: &mut crate::arena::Arena<RNode>, node: Key) {
    let deps = match g.get_mut(node) {
        Some(n) => std::mem::take(&mut n.deps),
        None => return,
    };
    for (src, sub_index) in deps {
        let moved = match g.get_mut(src) {
            Some(s) => {
                let i = sub_index as usize;
                if i >= s.subs.len() {
                    continue;
                }
                s.subs.swap_remove(i);
                s.subs.get(i).copied().map(|(m, k)| (m, k, i))
            }
            None => None,
        };
        if let Some((m, k, i)) = moved
            && let Some(mn) = g.get_mut(m)
            && let Some(e) = mn.deps.get_mut(k as usize)
        {
            e.1 = i as u32;
        }
    }
}

/// Remove `sub.deps[index]` (whose source is being disposed) keeping read order.
fn remove_dep_entry(g: &mut crate::arena::Arena<RNode>, sub: Key, index: usize) {
    let Some(n) = g.get_mut(sub) else { return };
    if index >= n.deps.len() {
        return;
    }
    n.deps.remove(index);
    // Fix back-indices of the following deps in their sources.
    let fix: Vec<(Key, u32, u32)> = n.deps[index..]
        .iter()
        .enumerate()
        .map(|(j, (src, si))| (*src, *si, (index + j) as u32))
        .collect();
    for (src, si, new_dep_index) in fix {
        if let Some(s) = g.get_mut(src)
            && let Some(e) = s.subs.get_mut(si as usize)
        {
            e.1 = new_dep_index;
        }
    }
}

pub(crate) fn is_memo<T: 'static>(cell: &Rc<dyn Any>) -> bool {
    cell.is::<MemoCell<T>>()
}

/// Read helpers for typed handles.
pub(crate) fn source_cell<T: 'static>(cell: &Rc<dyn Any>) -> Option<&SourceCell<T>> {
    cell.downcast_ref::<SourceCell<T>>()
}

impl Runtime {
    /// Typed value cell of a source node (internal fast paths).
    pub(crate) fn source_rc<T: 'static>(&self, key: Key) -> Option<Rc<SourceCell<T>>> {
        let cell = self.graph.borrow().get(key).map(|n| n.cell.clone())?;
        cell.downcast::<SourceCell<T>>().ok()
    }

    /// Snapshot of node `key`, or `None` if it no longer exists.
    pub(crate) fn node_snapshot(&self, key: Key) -> Option<NodeInfo> {
        let g = self.graph.borrow();
        let n = g.get(key)?;
        Some(NodeInfo {
            kind: n.kind,
            ty: n.ty,
            at: n.at,
            written: n.written,
            deps: n.deps.iter().map(|(d, _)| NodeId(*d)).collect(),
        })
    }
}
