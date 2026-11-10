//! Role: Shader clocks: which painted shaders need frames, and when the next is.
//!
//! Invariants:
//! - A node needs frames only while it is drawn with a shader whose clock
//!   runs: not `Still`, and not under reduced motion unless the shader is
//!   essential.
//! - The nodes with a shader clock come from `MotionSources`, and their
//!   drawn state is read at a paint list rebuild (`Model::refresh_shader_clocks`).
//! - At each tick every such node is repainted (its time advanced), and the
//!   next frame is the earliest of the nodes' next times: a display clock
//!   wants every tick, an `Hz(r)` clock wants the tick after `⌊now·r⌋`.

use std::collections::BTreeMap;

use kivact_types::NodeId;

use super::state::ClockNeed;
use crate::model::Model;

impl Model {
    /// Rebuild the clock needs of the drawn nodes at a paint list rebuild, and
    /// request the next frame they need.
    pub(crate) fn refresh_shader_clocks(&mut self) {
        let reduced = self.frame.reduced_motion();
        let needs: BTreeMap<NodeId, ClockNeed> = self
            .sources
            .shaders()
            .filter(|(id, _)| self.tree.drawn(*id))
            .filter_map(|(id, need)| Some((id, if reduced { need.reduced? } else { need.moving })))
            .collect();
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
