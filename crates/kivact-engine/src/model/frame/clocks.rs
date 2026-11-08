//! Role: Shader clocks: which painted shaders need frames, and when the next is.
//!
//! Invariants:
//! - A node needs frames only while it is drawn with a shader whose clock
//!   runs: not `Still`, and not under reduced motion unless the shader is
//!   essential.
//! - At each tick every such node is repainted (its time advanced), and the
//!   next frame is the earliest of the nodes' next times: a display clock
//!   wants every tick, an `Hz(r)` clock wants the tick after `⌊now·r⌋`.

use std::collections::BTreeMap;

use kivact_types::paint::{Fill, FillImage, FilterOp, ShaderClock, ShaderSpec};
use kivact_types::{DrawCmd, DrawList, NodeId};

use super::state::ClockNeed;
use crate::model::Model;

impl Model {
    /// Rebuild the clock needs of the drawn nodes at a paint list rebuild, and
    /// request the next frame they need.
    pub(crate) fn sync_shader_clocks(&mut self) {
        let reduced = self.frame.reduced_motion();
        let mut needs = BTreeMap::new();
        for (id, node) in self.tree.iter() {
            if !self.tree.drawn(id) {
                continue;
            }
            if let Some(need) = node_clock(node, reduced) {
                needs.insert(id, need);
            }
        }
        self.frame.shader_clocks = needs;
        self.request_clock_frames(self.frame.now());
    }

    /// At a tick at `now`: repaint the nodes whose shaders run on a clock and
    /// request the next frame they need.
    pub(crate) fn step_shader_clocks(&mut self, now: u64) {
        let ids: Vec<NodeId> = self.frame.shader_clocks.keys().copied().collect();
        for id in ids {
            self.repaint(id);
        }
        self.request_clock_frames(now);
    }

    /// Request the earliest next frame of the clock needs, after `now`.
    fn request_clock_frames(&mut self, now: u64) {
        let period = self.frame.period_ns();
        let next = self
            .frame
            .shader_clocks
            .values()
            .map(|need| match *need {
                ClockNeed::Display => now.saturating_add(period),
                ClockNeed::Hz(rate) => hz_next(now, rate),
            })
            .min();
        if let Some(at) = next {
            self.frame.request(at);
        }
    }
}

/// The host time (ns) of the first tick after `now` on an `rate` Hz clock:
/// the `(⌊now·rate / 1e9⌋ + 1)`-th tick.
fn hz_next(now: u64, rate: f64) -> u64 {
    let ticks = (now as f64 * rate / 1e9).floor() + 1.0;
    let at = ticks * 1e9 / rate;
    if at.is_finite() && at >= 0.0 {
        at as u64
    } else {
        now.saturating_add(1)
    }
}

/// The clock need of `node`'s shaders: its background, its filters and
/// backdrop, and the shaders of its draw scenes.
fn node_clock(node: &crate::tree::node::HostNode, reduced: bool) -> Option<ClockNeed> {
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
