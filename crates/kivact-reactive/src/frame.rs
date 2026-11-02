//! Role: Copy execution contexts and runtime-owned dependency collectors.
//!
//! Each guard restores the previous App and context on unwind. Only a fresh
//! computation owns a collector; pure copies share its slot, and untracked
//! or callback contexts suspend tracking without releasing outer collectors.
use crate::AppId;
use crate::arena::Key;
use crate::data::{Collector, Ctx, FrameGuard, Runtime};
use crate::error::{ErrorKind, misuse_at};
use crate::execution::{enter_app, executing_app, leave_app};
use crate::registry::current_runtime;
use std::panic::Location;

impl Runtime {
    #[inline]
    fn context(&self) -> Ctx {
        self.frames.borrow().last().copied().unwrap_or_default()
    }

    #[inline]
    fn push_frame(&self, context: Ctx) -> FrameGuard<'_> {
        let mut frames = self.frames.borrow_mut();
        frames.push(context);
        FrameGuard {
            rt: self,
            depth: frames.len(),
            outer_app: enter_app(self.id),
            collector: None,
        }
    }

    pub(crate) fn setup_frame(&self, owner: Key) -> FrameGuard<'_> {
        self.push_frame(Ctx {
            owner: Some(owner),
            setup: Some(owner),
            ..Ctx::default()
        })
    }

    pub(crate) fn tracking_frame(
        &self,
        node: Key,
        owner: Key,
        pure: bool,
        effect: bool,
    ) -> FrameGuard<'_> {
        let slot = {
            let mut collectors = self.collectors.borrow_mut();
            let slot = collectors.len();
            collectors.push(Collector {
                node,
                deps: Vec::new(),
                set: None,
            });
            slot
        };
        let mut guard = self.push_frame(Ctx {
            owner: Some(owner),
            setup: None,
            pure,
            effect: effect.then_some(node),
            tracking: Some(slot),
        });
        guard.collector = Some(slot);
        guard
    }

    pub(crate) fn pure_section(&self) -> FrameGuard<'_> {
        self.push_frame(Ctx {
            pure: true,
            ..self.context()
        })
    }

    pub(crate) fn untracked_frame(&self) -> FrameGuard<'_> {
        self.push_frame(Ctx {
            tracking: None,
            ..self.context()
        })
    }

    pub(crate) fn callback_frame(&self, owner: Key) -> FrameGuard<'_> {
        self.push_frame(Ctx {
            owner: Some(owner),
            tracking: None,
            ..self.context()
        })
    }

    pub(crate) fn handler_frame(&self, owner: Option<Key>) -> FrameGuard<'_> {
        self.push_frame(Ctx {
            owner,
            ..Ctx::default()
        })
    }

    pub(crate) fn current_owner(&self) -> Option<Key> {
        let context = self.context();
        context.owner.or(context.setup)
    }
    pub(crate) fn setup_owner(&self) -> Option<Key> {
        self.context().setup
    }
    pub(crate) fn current_effect(&self) -> Option<Key> {
        self.context().effect
    }
    pub(crate) fn is_pure(&self) -> bool {
        self.context().pure
    }

    #[inline]
    pub(crate) fn track(&self, node: Key) {
        let Some(slot) = self.context().tracking else {
            return;
        };
        let mut collectors = self.collectors.borrow_mut();
        let collector = &mut collectors[slot];
        if collector.node == node {
            return;
        }
        match &mut collector.set {
            Some(set) => {
                if set.insert(node) {
                    collector.deps.push(node);
                }
            }
            None => {
                if !collector.deps.contains(&node) {
                    collector.deps.push(node);
                    if collector.deps.len() >= 16 {
                        collector.set = Some(collector.deps.iter().copied().collect());
                    }
                }
            }
        }
    }

    /// Only actual computations contribute: copied contexts add no cycle entries.
    pub(crate) fn frame_path(&self) -> Vec<&'static Location<'static>> {
        let collectors = self.collectors.borrow();
        let graph = self.graph.borrow();
        collectors
            .iter()
            .filter_map(|collector| graph.get(collector.node).map(|node| node.at))
            .collect()
    }

    pub(crate) fn require_setup(&self, api: &'static str, at: &'static Location<'static>) -> Key {
        match self.setup_owner() {
            Some(owner) if self.owners.borrow().contains(owner) => owner,
            _ => misuse_at(
                ErrorKind::SetupOnly { api },
                format!("`{api}` can only be called during component setup"),
                at,
            ),
        }
    }
    pub(crate) fn require_not_pure(&self, api: &'static str, at: &'static Location<'static>) {
        if self.is_pure() {
            misuse_at(
                ErrorKind::WriteInPureContext,
                format!("`{api}` has side effects and cannot run inside a memo or render effect"),
                at,
            );
        }
    }
}

impl FrameGuard<'_> {
    #[inline]
    pub(crate) fn take_deps(&self) -> Vec<Key> {
        let Some(slot) = self.collector else {
            return Vec::new();
        };
        std::mem::take(&mut self.rt.collectors.borrow_mut()[slot].deps)
    }
}
impl Drop for FrameGuard<'_> {
    #[inline]
    fn drop(&mut self) {
        if let Some(slot) = self.collector {
            let mut collectors = self.rt.collectors.borrow_mut();
            debug_assert_eq!(
                collectors.len(),
                slot + 1,
                "invariant: frame guards drop in LIFO order"
            );
            collectors.truncate(slot);
        }
        let mut frames = self.rt.frames.borrow_mut();
        debug_assert_eq!(
            frames.len(),
            self.depth,
            "invariant: frame guards drop in LIFO order"
        );
        frames.truncate(self.depth - 1);
        drop(frames);
        leave_app(self.outer_app);
    }
}

/// Run `f` without tracking reads. Purity (memo/render context) is inherited,
/// so writes inside `untrack` in a memo are still rejected.
pub fn untrack<R>(f: impl FnOnce() -> R) -> R {
    // Outside every App nothing is tracked: no frame is needed.
    if executing_app() == AppId::NONE {
        return f();
    }
    match current_runtime() {
        Some(rt) => {
            let _g = rt.untracked_frame();
            f()
        }
        None => f(),
    }
}

/// Run `f` as a pure computation: a write or a side effect inside it (a
/// signal write, `cancel`, `focus`, a platform request, ...) is a located
/// `WriteInPureContext`. Tracking, the setup owner and the running effect
/// stay those of the caller, and the previous purity is restored also on
/// unwind. Layers use it for closures that only compute a value (flow
/// conditions, list items and keys, resource sources) inside an effect that
/// may write. Outside every App it only calls `f`.
pub fn pure<R>(f: impl FnOnce() -> R) -> R {
    match current_runtime() {
        Some(rt) => {
            let _pure = rt.pure_section();
            f()
        }
        None => f(),
    }
}

/// Require component setup: a located `SetupOnly` naming `api` outside
/// setup (also outside every App). It neither chooses an owner nor attaches
/// work to the App's root.
#[track_caller]
pub fn require_setup(api: &'static str) {
    crate::scope::setup_scope(api, Location::caller());
}

/// Require a context that may have side effects: a located
/// `WriteInPureContext` naming `api` in a memo, a render effect or a
/// [`pure`] section. It changes nothing otherwise.
#[track_caller]
pub fn require_effectful(api: &'static str) {
    if let Some(rt) = current_runtime() {
        rt.require_not_pure(api, Location::caller());
    }
}
