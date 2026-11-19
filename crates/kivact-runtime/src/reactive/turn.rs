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
        let (_, key) = self.render.pop_first()?;
        Self::unmark(graph, key);
        Some(key)
    }

    /// Every queued Render/Internal effect in order, removed.
    pub(crate) fn take_render(
        &mut self,
        graph: &mut crate::reactive::arena::Arena<RNode>,
    ) -> Vec<Key> {
        let set = std::mem::take(&mut self.render);
        set.into_iter()
            .map(|(_, k)| {
                Self::unmark(graph, k);
                k
            })
            .collect()
    }

    /// Every queued User effect in order, removed.
    pub(crate) fn take_user(
        &mut self,
        graph: &mut crate::reactive::arena::Arena<RNode>,
    ) -> Vec<Key> {
        let set = std::mem::take(&mut self.user);
        set.into_iter()
            .map(|(_, k)| {
                Self::unmark(graph, k);
                k
            })
            .collect()
    }

    /// An effect taken out of its queue set is stale again, not queued.
    fn unmark(graph: &mut crate::reactive::arena::Arena<RNode>, key: Key) {
        if let Some(n) = graph.get_mut(key)
            && matches!(n.data, NodeData::Effect(_) | NodeData::Watcher(_))
            && let CompState::Queued(stale) = n.state
        {
            n.state = CompState::Stale(stale);
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
    /// per graph node (plus one), counted against the graph as it is at
    /// each run: an effect may mount many nodes, so the bound grows during
    /// the pass, and a large App whose effects each run a few times is not
    /// cut. Either cut is reported as a `Cycle` to the default sink.
    pub(crate) fn stabilize(&self) {
        let mut runs: usize = 0;
        loop {
            let next = self
                .sched
                .borrow_mut()
                .pop_render(&mut self.graph.borrow_mut());
            let Some(key) = next else { break };
            runs += 1;
            let own = self.count_pass_run(key);
            let limit = CYCLE_LIMIT * (self.graph.borrow().len() + 1);
            if usize::from(own) > CYCLE_LIMIT || runs > limit {
                self.abandon_render_effects(key);
                let at = self.node_at(key);
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
        self.reset_pass_runs();
    }

    /// Count one run of effect `key` in this stabilize pass; returns the count.
    fn count_pass_run(&self, key: Key) -> u8 {
        let mut g = self.graph.borrow_mut();
        let Some(NodeData::Effect(e) | NodeData::Watcher(e)) = g.get_mut(key).map(|n| &mut n.data)
        else {
            return 0;
        };
        if e.pass_runs == 0 {
            self.pass_touched.borrow_mut().push(key);
        }
        e.pass_runs = e.pass_runs.saturating_add(1);
        e.pass_runs
    }

    /// Forget the per-pass run counts of the effects that ran.
    fn reset_pass_runs(&self) {
        let mut touched = self.pass_touched.borrow_mut();
        let mut g = self.graph.borrow_mut();
        for key in touched.drain(..) {
            if let Some(NodeData::Effect(e) | NodeData::Watcher(e)) =
                g.get_mut(key).map(|n| &mut n.data)
            {
                e.pass_runs = 0;
            }
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
            if let Some(n) = g.get_mut(k)
                && n.is_computation()
            {
                n.state.abandon();
            }
        }
    }

    /// Run the user effects queued so far (one round). Returns whether any ran.
    pub(crate) fn run_user_effects(&self) -> bool {
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
            if let Some(n) = g.get_mut(*k)
                && n.is_computation()
            {
                n.state.abandon();
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

#[cfg(test)]
mod tests {
    //! Role: One stabilisation pass is bounded by the graph it runs over, and
    //! a self-rerunning effect is still a cycle.

    use std::cell::Cell;
    use std::panic::Location;
    use std::rc::Rc;

    use crate::reactive::data::{EffectKind, Runtime};
    use crate::reactive::error::ErrorKind;
    use crate::reactive::host_types::Host;

    struct Quiet;

    impl Host for Quiet {
        fn wake(&self) {}
    }

    /// How many `Cycle` errors reached the default sink.
    fn cycles(rt: &Runtime) -> usize {
        rt.errors()
            .iter()
            .filter(|e| matches!(e.kind(), ErrorKind::Cycle { .. }))
            .count()
    }

    #[test]
    fn a_pass_that_mounts_three_thousand_effects_is_not_a_cycle() {
        const MOUNTED: usize = 3_000;
        let rt = Runtime::register(
            Rc::new(Quiet),
            0,
            crate::TaskState::new(None, crate::ThreadWaker::new(|| {})),
        );
        let at = Location::caller();
        assert_eq!(rt.create_owner(None, at), rt.root);
        rt.enter(|| {
            let root = rt.root();
            let go = root.signal(0u32, at);
            let bus = root.signal(0u32, at);
            // Each mounted effect counts its runs: one at mount, one in the pass.
            let runs: Rc<Vec<Cell<u32>>> = Rc::new((0..MOUNTED).map(|_| Cell::new(0)).collect());
            let driver_runs = Rc::clone(&runs);
            // The graph is small here; the driver grows it during the pass.
            root.effect(
                EffectKind::Internal,
                move || {
                    if go.get() == 0 {
                        return;
                    }
                    for i in 0..MOUNTED {
                        let runs = Rc::clone(&driver_runs);
                        root.effect(
                            EffectKind::Internal,
                            move || {
                                let _ = bus.get();
                                let c = &runs[i];
                                c.set(c.get() + 1);
                            },
                            at,
                        );
                    }
                    bus.set(1);
                },
                at,
            );
            go.set(1);
            rt.stabilize();

            assert_eq!(cycles(&rt), 0, "a pass that mounts effects is not a cycle");
            let short = runs.iter().filter(|c| c.get() != 2).count();
            assert_eq!(
                short, 0,
                "every mounted effect runs once at mount and once in the pass"
            );
        });
        rt.shutdown();
    }

    #[test]
    fn an_effect_that_reruns_itself_is_a_cycle() {
        let rt = Runtime::register(
            Rc::new(Quiet),
            0,
            crate::TaskState::new(None, crate::ThreadWaker::new(|| {})),
        );
        let at = Location::caller();
        assert_eq!(rt.create_owner(None, at), rt.root);
        rt.enter(|| {
            let root = rt.root();
            let kick = root.signal(0u32, at);
            let s = root.signal(0u32, at);
            root.effect(
                EffectKind::Internal,
                move || {
                    let _ = kick.get();
                    let v = s.get();
                    s.set(v + 1);
                },
                at,
            );
            kick.set(1);
            rt.stabilize();

            assert_eq!(cycles(&rt), 1, "an effect that writes what it reads is cut");
        });
        rt.shutdown();
    }
}
