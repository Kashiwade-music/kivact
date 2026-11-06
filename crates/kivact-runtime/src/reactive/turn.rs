//! Role: Effect scheduling within a turn.
//!
//! Render and Internal effects run during stabilisation in creation order
//! (owners before the effects they create, so flows dispose children before
//! those children would run); User effects run in rounds the driver of the
//! turn asks for (after it took the output of stabilisation). The order of a
//! whole turn (events, timers and tasks, commit, `on_mount` and User
//! effects) is the client's: see `kivact_core::client`.
//!
//! Invariants:
//! - One effect running more than [`CYCLE_LIMIT`] (100) times in one
//!   stabilisation, or the pass exceeding that limit per graph node, is cut
//!   and reported as a `Cycle` to the default sink; the cut effects become
//!   clean so a later change schedules them again.

use crate::reactive::arena::Key;
use crate::reactive::data::{CYCLE_LIMIT, CompState, EffectKind, NodeData, RNode, Runtime, Sched};
use crate::reactive::error::{ErrorKind, new_error};

impl Sched {
    /// Queue `key`; returns whether it was not queued yet.
    pub(crate) fn push(
        &mut self,
        state: &mut CompState,
        kind: EffectKind,
        seq: u64,
        key: Key,
    ) -> bool {
        state.enqueue(self, kind, seq, key)
    }

    /// The first queued Render/Internal effect, removed.
    pub(crate) fn pop_render(
        &mut self,
        graph: &mut crate::reactive::arena::Arena<RNode>,
    ) -> Option<Key> {
        let (_, key) = self.render.first().copied()?;
        Self::unqueue(self, graph, key);
        Some(key)
    }

    /// Every queued Render/Internal effect in order, removed.
    pub(crate) fn take_render(
        &mut self,
        graph: &mut crate::reactive::arena::Arena<RNode>,
    ) -> Vec<Key> {
        self.render
            .clone()
            .into_iter()
            .map(|(_, k)| {
                Self::unqueue(self, graph, k);
                k
            })
            .collect()
    }

    /// Every queued User effect in order, removed.
    pub(crate) fn take_user(
        &mut self,
        graph: &mut crate::reactive::arena::Arena<RNode>,
    ) -> Vec<Key> {
        self.user
            .clone()
            .into_iter()
            .map(|(_, k)| {
                Self::unqueue(self, graph, k);
                k
            })
            .collect()
    }

    fn unqueue(&mut self, graph: &mut crate::reactive::arena::Arena<RNode>, key: Key) {
        if let Some(NodeData::Effect(e) | NodeData::Watcher(e)) =
            graph.get_mut(key).map(|n| &mut n.data)
        {
            e.run.state.dequeue(self, e.kind, e.seq, key);
        }
    }
    pub(crate) fn remove(&mut self, state: &mut CompState, kind: EffectKind, seq: u64, key: Key) {
        state.dequeue(self, kind, seq, key);
    }
    pub(crate) fn has_render(&self) -> bool {
        !self.render.is_empty()
    }

    pub(crate) fn has_user(&self) -> bool {
        !self.user.is_empty()
    }
}

impl CompState {
    fn enqueue(&mut self, sched: &mut Sched, kind: EffectKind, seq: u64, key: Key) -> bool {
        let Self::Stale(stale) = *self else {
            return false;
        };
        match kind {
            EffectKind::User => sched.user.insert((seq, key)),
            _ => sched.render.insert((seq, key)),
        };
        *self = Self::Queued(stale);
        true
    }
    fn dequeue(&mut self, sched: &mut Sched, kind: EffectKind, seq: u64, key: Key) {
        let Self::Queued(stale) = *self else { return };
        match kind {
            EffectKind::User => sched.user.remove(&(seq, key)),
            _ => sched.render.remove(&(seq, key)),
        };
        *self = Self::Stale(stale);
    }
    fn abandon(&mut self) {
        *self = Self::Clean;
    }
}

impl Runtime {
    /// Run queued Render/Internal effects until none is left. One effect
    /// running more than `CYCLE_LIMIT` times in one pass is a cycle; that
    /// limit is fixed. The pass as a whole stops after `CYCLE_LIMIT` runs
    /// per graph node (plus one): that bound grows with the size of the
    /// App, so a large App whose effects each run a few times is not cut.
    /// Either cut is reported as a `Cycle` to the default sink.
    pub fn stabilize(&self) {
        let mut runs: usize = 0;
        let limit = CYCLE_LIMIT * (self.graph.borrow().len() + 1);
        let epoch = self.stab_epoch.get() + 1;
        self.stab_epoch.set(epoch);
        loop {
            let next = self
                .sched
                .borrow_mut()
                .pop_render(&mut self.graph.borrow_mut());
            let Some(key) = next else { break };
            runs += 1;
            let own_runs = match self.graph.borrow_mut().get_mut(key).map(|n| &mut n.data) {
                Some(NodeData::Effect(e) | NodeData::Watcher(e)) => {
                    e.pass_runs.note_run(epoch).is_err()
                }
                _ => false,
            };
            if own_runs || runs > limit {
                self.abandon_render_effects(key);
                let at = self.graph.borrow().get(key).map(|n| n.at);
                self.default_sink(new_error(
                    ErrorKind::Cycle {
                        path: at.into_iter().collect(),
                    },
                    "stabilisation did not converge: an effect exceeded the per-pass run limit",
                    at,
                ));
                break;
            }
            self.run_effect(key);
        }
    }

    /// Drop the queued Render/Internal effects after a cycle. They become
    /// clean so that a later change schedules them again.
    fn abandon_render_effects(&self, current: Key) {
        let mut keys = self
            .sched
            .borrow_mut()
            .take_render(&mut self.graph.borrow_mut());
        keys.push(current);
        let mut g = self.graph.borrow_mut();
        for k in keys {
            if let Some(c) = g.get_mut(k).and_then(RNode::computation_mut) {
                c.state.abandon();
            }
        }
    }

    /// Run the user effects queued so far (one round). Returns whether any ran.
    pub fn run_user_effects(&self) -> bool {
        let batch = self
            .sched
            .borrow_mut()
            .take_user(&mut self.graph.borrow_mut());
        let ran = !batch.is_empty();
        for k in batch {
            self.run_effect(k);
        }
        ran
    }

    /// Drop pending user effects after a cycle was detected.
    pub(crate) fn abandon_user_queue(&self) -> Vec<Key> {
        let v = self
            .sched
            .borrow_mut()
            .take_user(&mut self.graph.borrow_mut());
        let mut g = self.graph.borrow_mut();
        for k in &v {
            if let Some(c) = g.get_mut(*k).and_then(RNode::computation_mut) {
                c.state.abandon();
            }
        }
        v
    }

    /// Whether user effects are queued.
    pub(crate) fn has_user_effects(&self) -> bool {
        self.sched.borrow().has_user()
    }

    /// Whether Render/Internal effects are queued.
    pub(crate) fn has_render_effects(&self) -> bool {
        self.sched.borrow().has_render()
    }
}
