//! Role: Drawables in the model: the mounted drawable roots, what the app asks of each export, and its retries.
//!
//! A drawable root is a node whose properties carry a `DrawableRole`. The
//! model keeps one entry per such root, reconciled at each commit: an
//! entry follows its role and goes when its root is removed. The app's
//! requests (`HostRequest::DrawableExport`, `SetDrawableGeometry`) are kept
//! by drawable id, so a request made before the drawable is mounted still
//! applies once it is. `place` (in `drawables/place.rs`) decides where a
//! canvas draws each drawable.
//!
//! Invariants:
//! - A geometry request whose generation is not the current ask's
//!   generation is dropped (an older request's geometry never shows).
//! - `ExportRetry` is decided per presentation only, from that
//!   presentation's pending exports; a retry is a deadline on the pump's
//!   clock, never an `Instant`.

use std::collections::{BTreeMap, BTreeSet};

use kivact_types::geometry3d::Projective;
use kivact_types::{DrawableId, NodeId};

use crate::contract::{DrawableMode, ExportRequestMode, HostRequest};
use crate::model::Model;

pub(crate) mod hit;
pub(crate) mod place;

/// The longest interval a pending export is retried at (ns).
pub(crate) const RETRY_MAX_NS: u64 = 100_000_000;

/// The export the app asked of one drawable.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExportAsk {
    /// The generation of the app's request.
    pub(crate) generation: u64,
    /// How the frames are delivered.
    pub(crate) mode: ExportRequestMode,
    /// Device px per logical px of the frames.
    pub(crate) scale: f64,
}

/// When a presentation skipped a drawable's export, the next attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportRetry {
    /// Nothing is waiting.
    Idle,
    /// The last presentation skipped the export: retry at `retry_at_ns`
    /// (the host clock's ns), `interval_ns` apart from the last try.
    Pending { retry_at_ns: u64, interval_ns: u64 },
}

impl ExportRetry {
    /// Decide the state from one presentation: `pending` when it skipped
    /// this drawable's export. `period_ns` is one display frame.
    pub(crate) fn presented(&mut self, pending: bool, now_ns: u64, period_ns: u64) {
        *self = match (pending, *self) {
            (false, _) => Self::Idle,
            (true, Self::Idle) => Self::Pending {
                retry_at_ns: now_ns + period_ns,
                interval_ns: period_ns,
            },
            (true, Self::Pending { interval_ns, .. }) => {
                let interval_ns = interval_ns.saturating_mul(2).min(RETRY_MAX_NS);
                Self::Pending {
                    retry_at_ns: now_ns + interval_ns,
                    interval_ns,
                }
            }
        };
    }

    /// The deadline of the pending retry, if any.
    pub(crate) fn deadline(&self) -> Option<u64> {
        match self {
            Self::Idle => None,
            Self::Pending { retry_at_ns, .. } => Some(*retry_at_ns),
        }
    }
}

/// One mounted drawable root.
pub(crate) struct DrawableEntry {
    /// The root node of the drawable's content.
    pub(crate) root: NodeId,
    /// Whether the canvas draws it or it is exported.
    pub(crate) mode: DrawableMode,
    /// The retry of a skipped export.
    pub(crate) retry: ExportRetry,
}

/// The drawable roots of the model and the app's requests, by id.
#[derive(Default)]
pub(crate) struct Drawables {
    /// The mounted roots.
    entries: BTreeMap<DrawableId, DrawableEntry>,
    /// The app's current export request of each drawable.
    asks: BTreeMap<DrawableId, ExportAsk>,
    /// The app's geometry for the generation it names (`None` hides it).
    geometry: BTreeMap<DrawableId, (u64, Option<Projective>)>,
    /// The GPU export pools the window has made, by drawable: generation and size.
    pools: BTreeMap<DrawableId, (u64, (u32, u32))>,
    /// Nodes whose drawable role was written since the last reconcile.
    // xtask-allow(state-fields): Dirty set of role writes; reconcile drains it at the commit.
    noted: BTreeSet<NodeId>,
}

/// Apply one drawable request of the app (`HostRequest::DrawableExport` or
/// `SetDrawableGeometry`); any other request is a caller's mistake.
pub(crate) fn apply_request(m: &mut Model, request: HostRequest) {
    match request {
        HostRequest::DrawableExport {
            id,
            generation,
            mode,
            scale,
        } => m.drawables.ask(
            id,
            ExportAsk {
                generation,
                mode,
                scale,
            },
        ),
        HostRequest::SetDrawableGeometry {
            id,
            generation,
            geometry,
        } => m
            .drawables
            .set_geometry(id, generation, geometry.map(|g| g.flatten())),
        _ => debug_assert!(false, "only drawable requests reach the drawables"),
    }
}

/// A GPU export pool a drawable wants this frame.
pub(crate) struct WantedPool {
    /// The drawable.
    pub(crate) id: DrawableId,
    /// The app's request generation the pool serves.
    pub(crate) generation: u64,
    /// The window's adapter the textures are shared on.
    pub(crate) adapter: kivact_types::GpuAdapter,
    /// The pool's size in device px.
    pub(crate) size: (u32, u32),
}

/// The export of an exported drawable's layer `layer`, sized from its box
/// (`bounds`, logical px) at the app's scale.
pub(crate) fn export_of(
    sink: DrawableId,
    layer: u32,
    bounds: kivact_types::Rect,
    ask: &ExportAsk,
) -> kivact_types::render::Export {
    let px = |v: f64| (v * ask.scale).ceil().max(1.0) as u32;
    kivact_types::render::Export {
        layer,
        sink,
        size: (px(bounds.width), px(bounds.height)),
        scale: ask.scale,
        generation: ask.generation,
        mode: match ask.mode {
            ExportRequestMode::Gpu(_) => kivact_types::render::ExportMode::Gpu,
            _ => kivact_types::render::ExportMode::Pixels,
        },
    }
}

impl Model {
    /// The exports to compose this frame: each exported drawable the app asked
    /// frames of, with its root node and request. A GPU request waits for its
    /// pool (`pool_of`) of the same generation.
    pub(crate) fn export_requests(&self) -> Vec<(DrawableId, NodeId, ExportAsk)> {
        self.drawables
            .iter()
            .filter(|(_, entry)| entry.mode == DrawableMode::Export)
            .filter_map(|(id, entry)| {
                let ask = self.drawables.ask_of(id)?.clone();
                let ready = match ask.mode {
                    ExportRequestMode::None => false,
                    ExportRequestMode::Pixels => true,
                    ExportRequestMode::Gpu(_) => self
                        .drawables
                        .pool_of(id)
                        .is_some_and(|(generation, _)| generation == ask.generation),
                };
                ready.then_some((id, entry.root, ask))
            })
            .collect()
    }

    /// The GPU export pools wanted now: every mounted exported drawable whose
    /// app asked for textures, sized from its laid-out box in device px.
    pub(crate) fn wanted_pools(&self) -> Vec<WantedPool> {
        self.drawables
            .iter()
            .filter(|(_, entry)| entry.mode == DrawableMode::Export)
            .filter_map(|(id, entry)| {
                let ask = self.drawables.ask_of(id)?;
                let ExportRequestMode::Gpu(adapter) = &ask.mode else {
                    return None;
                };
                let rect = self.rect(entry.root)?;
                let px = |v: f64| (v * ask.scale).ceil().max(1.0) as u32;
                Some(WantedPool {
                    id,
                    generation: ask.generation,
                    adapter: adapter.clone(),
                    size: (px(rect.width), px(rect.height)),
                })
            })
            .collect()
    }

    /// Make the entries follow the roles written since the last commit, and
    /// drop those whose root left the tree.
    pub(crate) fn reconcile_drawables(&mut self) {
        for node in self.drawables.take_noted() {
            let role = self
                .tree
                .get(node)
                .and_then(|n| n.properties.drawable.map(|r| (r.id, r.mode)));
            self.drawables.forget_root(node);
            if let Some((id, mode)) = role {
                self.drawables.follow(id, node, mode);
            }
        }
        let tree = &self.tree;
        self.drawables.retain_roots(|root| tree.get(root).is_some());
    }
}

impl Drawables {
    /// Record that `node`'s drawable role was written.
    pub(crate) fn note(&mut self, node: NodeId) {
        self.noted.insert(node);
    }

    /// The nodes whose roles changed, drained.
    pub(crate) fn take_noted(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.noted).into_iter().collect()
    }

    /// The entry of `id`.
    pub(crate) fn get(&self, id: DrawableId) -> Option<&DrawableEntry> {
        self.entries.get(&id)
    }

    /// Every entry, in id order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (DrawableId, &DrawableEntry)> + '_ {
        self.entries.iter().map(|(id, entry)| (*id, entry))
    }

    /// The app's current export request of `id`.
    pub(crate) fn ask_of(&self, id: DrawableId) -> Option<&ExportAsk> {
        self.asks.get(&id)
    }

    /// The geometry of `id` for its current request, if the app set one.
    pub(crate) fn geometry_of(&self, id: DrawableId) -> Option<Projective> {
        let (generation, geometry) = self.geometry.get(&id)?;
        let ask = self.asks.get(&id)?;
        (ask.generation == *generation)
            .then_some(*geometry)
            .flatten()
    }

    /// Make the entry of `id` follow its root `root` with `mode`.
    pub(crate) fn follow(&mut self, id: DrawableId, root: NodeId, mode: DrawableMode) {
        let entry = self.entries.entry(id).or_insert(DrawableEntry {
            root,
            mode,
            retry: ExportRetry::Idle,
        });
        entry.root = root;
        entry.mode = mode;
    }

    /// Drop the entries whose root `alive` no longer holds.
    pub(crate) fn retain_roots(&mut self, alive: impl Fn(NodeId) -> bool) {
        self.entries.retain(|_, entry| alive(entry.root));
    }

    /// Drop the entries whose root is `root` (its role was removed or replaced).
    pub(crate) fn forget_root(&mut self, root: NodeId) {
        self.entries.retain(|_, entry| entry.root != root);
    }

    /// The app asked `id` for an export. A new generation starts its retry afresh.
    pub(crate) fn ask(&mut self, id: DrawableId, ask: ExportAsk) {
        if self
            .asks
            .get(&id)
            .is_none_or(|old| old.generation != ask.generation)
            && let Some(entry) = self.entries.get_mut(&id)
        {
            entry.retry = ExportRetry::Idle;
        }
        self.asks.insert(id, ask);
    }

    /// The GPU pool the window made for `id`: its generation and size in px.
    pub(crate) fn pool_of(&self, id: DrawableId) -> Option<(u64, (u32, u32))> {
        self.pools.get(&id).copied()
    }

    /// Record the GPU pool the window made for `id`, or forget it (`None`).
    pub(crate) fn set_pool(&mut self, id: DrawableId, pool: Option<(u64, (u32, u32))>) {
        match pool {
            Some(pool) => self.pools.insert(id, pool),
            None => self.pools.remove(&id),
        };
    }

    /// The drawables whose GPU pool exists but is no longer wanted.
    pub(crate) fn unwanted_pools(&self, wanted: &[DrawableId]) -> Vec<DrawableId> {
        self.pools
            .keys()
            .filter(|id| !wanted.contains(id))
            .copied()
            .collect()
    }

    /// The earliest retry deadline of a skipped export, if any.
    pub(crate) fn retry_deadline(&self) -> Option<u64> {
        self.entries
            .values()
            .filter_map(|entry| entry.retry.deadline())
            .min()
    }

    /// Whether a retry of a skipped export is due at `now_ns`.
    pub(crate) fn retry_due(&self, now_ns: u64) -> bool {
        self.retry_deadline().is_some_and(|at| at <= now_ns)
    }

    /// Decide every drawable's retry from one presentation: `pending` names the
    /// drawables whose export it skipped; the others are idle.
    pub(crate) fn presented(&mut self, pending: &[DrawableId], now_ns: u64, period_ns: u64) {
        for (id, entry) in self.entries.iter_mut() {
            entry
                .retry
                .presented(pending.contains(id), now_ns, period_ns);
        }
    }

    /// The app set the geometry of `id` for `generation`; a request of
    /// another generation is dropped.
    pub(crate) fn set_geometry(
        &mut self,
        id: DrawableId,
        generation: u64,
        geometry: Option<Projective>,
    ) {
        if self
            .asks
            .get(&id)
            .is_some_and(|ask| ask.generation == generation)
        {
            self.geometry.insert(id, (generation, geometry));
        }
    }
}
