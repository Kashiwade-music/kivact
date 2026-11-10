//! Role: The index of the props that ask for frames or layout motion.
//!
//! Invariants:
//! - A projection of the node props, written only by `Model::index_sources`
//!   (each touched node, at `Model::sync` through `Model::derive`) and
//!   `Model::forget_sources` (each node `Model::remove_now` destroys). It
//!   records neither whether a node is drawn nor whether motion is reduced:
//!   the readers filter by those when they read.
//! - The frame sources (`Frame::shader_clocks` and the `Playback` entries)
//!   are derived from it at a paint list rebuild only; everything that
//!   changes what that derivation reads restacks and redraws: an entry here
//!   changed, a node's drawn state (structure, `hidden`, `visibility`),
//!   reduced motion, a finished asset.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use kivact_types::paint::{Fill, FillImage, FilterOp, ShaderClock, ShaderSpec};
use kivact_types::{Asset, DrawCmd, DrawList, NodeId};

use super::state::ClockNeed;
use crate::tree::node::HostNode;

/// The clock a node's shaders need with and without reduced motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShaderNeed {
    /// Motion allowed: every running clock.
    pub(crate) moving: ClockNeed,
    /// Reduced motion: the essential shaders' clock only.
    pub(crate) reduced: Option<ClockNeed>,
}

#[derive(Default)]
pub(crate) struct MotionSources {
    shaders: BTreeMap<NodeId, ShaderNeed>,
    /// (node, slot) → the image that would play there (slot 0: content image).
    images: BTreeMap<(NodeId, u8), Asset>,
    layout_transitions: BTreeSet<NodeId>,
    layout_ids: BTreeMap<Arc<str>, BTreeSet<NodeId>>,
    /// The key each node is filed under in `layout_ids`.
    filed_ids: BTreeMap<NodeId, Arc<str>>,
}

impl MotionSources {
    /// File `id` from its node `n` (`None`: it is gone). Returns whether a
    /// frame source (a shader clock or an image) of it changed.
    pub(crate) fn file(&mut self, id: NodeId, n: Option<&HostNode>) -> bool {
        let shader = n.and_then(|n| {
            Some(ShaderNeed {
                moving: node_clock(n, false)?,
                reduced: node_clock(n, true),
            })
        });
        let mut changed = match shader {
            Some(s) => self.shaders.insert(id, s) != Some(s),
            None => self.shaders.remove(&id).is_some(),
        };
        let images = n
            .map(crate::model::playback::state::node_images)
            .unwrap_or_default();
        let old: Vec<(u8, Asset)> = self
            .images
            .range((id, 0)..=(id, u8::MAX))
            .map(|(k, a)| (k.1, a.clone()))
            .collect();
        if old != images {
            for (slot, _) in old {
                self.images.remove(&(id, slot));
            }
            for (slot, asset) in images {
                self.images.insert((id, slot), asset);
            }
            changed = true;
        }
        if n.is_some_and(|n| n.layout().is_some_and(|l| l.layout_transition.is_some())) {
            self.layout_transitions.insert(id);
        } else {
            self.layout_transitions.remove(&id);
        }
        let key = n.and_then(|n| n.layout_id().cloned());
        if self.filed_ids.get(&id) != key.as_ref() {
            if let Some(old) = self.filed_ids.remove(&id)
                && let Some(set) = self.layout_ids.get_mut(&old)
            {
                set.remove(&id);
                if set.is_empty() {
                    self.layout_ids.remove(&old);
                }
            }
            if let Some(key) = key {
                self.layout_ids.entry(key.clone()).or_default().insert(id);
                self.filed_ids.insert(id, key);
            }
        }
        changed
    }
    pub(crate) fn shaders(&self) -> impl Iterator<Item = (NodeId, ShaderNeed)> + '_ {
        self.shaders.iter().map(|(id, n)| (*id, *n))
    }
    pub(crate) fn images(&self) -> impl Iterator<Item = ((NodeId, u8), &Asset)> + '_ {
        self.images.iter().map(|(k, a)| (*k, a))
    }
    pub(crate) fn layout_transitions(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.layout_transitions.iter().copied()
    }
    pub(crate) fn has_layout_transition(&self, id: NodeId) -> bool {
        self.layout_transitions.contains(&id)
    }
    /// For each layout id carried by two or more nodes: the second-lowest
    /// node (the first one created after the original).
    pub(crate) fn duplicate_layout_ids(&self) -> impl Iterator<Item = (NodeId, &Arc<str>)> + '_ {
        self.layout_ids
            .iter()
            .filter_map(|(key, ids)| ids.iter().nth(1).map(|id| (*id, key)))
    }
    pub(crate) fn layout_id_nodes(&self, key: &str) -> impl Iterator<Item = NodeId> + '_ {
        self.layout_ids.get(key).into_iter().flatten().copied()
    }
}

/// The clock need of `node`'s shaders: its background, its filters and
/// backdrop, and the shaders of its draw scenes.
fn node_clock(node: &HostNode, reduced: bool) -> Option<ClockNeed> {
    let paint = node.paint();
    let mut need = paint
        .and_then(|p| p.background.as_ref())
        .and_then(|f| fill_clock(f, reduced));
    if let Some(effects) = paint.and_then(|p| p.effects.as_deref()) {
        need = combine(need, ops_clock(&effects.filter, reduced));
        need = combine(need, ops_clock(&effects.backdrop, reduced));
    }
    for scene in node.scenes().iter().flatten() {
        need = combine(need, scene_clock(scene, reduced));
    }
    need
}

/// The clock need of the shaders of a draw list.
fn scene_clock(scene: &DrawList, reduced: bool) -> Option<ClockNeed> {
    scene.cmds.iter().fold(None, |need, cmd| {
        let own = match cmd {
            DrawCmd::Rect { fill, .. } | DrawCmd::RoundedRect { fill, .. } => {
                fill_clock(fill, reduced)
            }
            DrawCmd::Shape(shape) => fill_clock(&shape.paint, reduced),
            DrawCmd::PushLayer(layer) => ops_clock(&layer.filter, reduced),
            _ => None,
        };
        combine(need, own)
    })
}

/// The clock need of the shader layers of `fill`.
fn fill_clock(fill: &Fill, reduced: bool) -> Option<ClockNeed> {
    let Fill::Layers(layers) = fill else {
        return None;
    };
    layers.iter().fold(None, |need, layer| match &layer.image {
        FillImage::Shader(spec) => combine(need, spec_clock(spec, reduced)),
        _ => need,
    })
}

/// The clock need of the shader filters among `ops`.
fn ops_clock(ops: &[FilterOp], reduced: bool) -> Option<ClockNeed> {
    ops.iter().fold(None, |need, op| match op {
        FilterOp::Shader(spec) => combine(need, spec_clock(spec, reduced)),
        _ => need,
    })
}

/// The clock need of one shader: none when it is still, or when reduced
/// motion stops it (unless it is essential).
fn spec_clock(spec: &ShaderSpec, reduced: bool) -> Option<ClockNeed> {
    if reduced && !spec.essential {
        return None;
    }
    match spec.clock {
        ShaderClock::Still => None,
        ShaderClock::Hz(rate) if rate.is_finite() && rate > 0.0 => Some(ClockNeed::Hz(rate)),
        ShaderClock::Hz(_) => None,
        ShaderClock::Display => Some(ClockNeed::Display),
    }
}

/// The more frequent of two needs (a display clock is the most frequent).
fn combine(a: Option<ClockNeed>, b: Option<ClockNeed>) -> Option<ClockNeed> {
    match (a, b) {
        (Some(ClockNeed::Display), _) | (_, Some(ClockNeed::Display)) => Some(ClockNeed::Display),
        (Some(ClockNeed::Hz(x)), Some(ClockNeed::Hz(y))) => Some(ClockNeed::Hz(x.max(y))),
        (x, None) => x,
        (None, y) => y,
    }
}
