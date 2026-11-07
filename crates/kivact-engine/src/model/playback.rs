//! Role: Which frame each drawn animated image shows, and when it changes.
//!
//! The content image of a node plays when it is drawn, its `playing` prop
//! is not false, reduced motion is off and its asset is animated. It plays
//! from the frame it was first drawn in, repeats its run (`Repeat`), and
//! then holds its last frame. Background layers are not played yet.
//!
//! Invariants:
//! - An entry exists only for a drawn image that plays; it is dropped when
//!   the image stops playing or its asset changes (then it starts again).
//! - Frame changes are ticks: each playing image asks for its next change
//!   and is repainted at it (`Model::step_playback`). Nothing is requested
//!   for a finished run.
//! - A frame is a pure function of the run's start and the time, so the
//!   paint never needs the store (`Model::playback_frame`).

use std::collections::BTreeMap;

use arvane::asset::AnimationInfo;
use kivact_types::motion::Repeat;
use kivact_types::{Asset, NodeId};

use crate::model::Model;

/// The playbacks of the animated images, keyed by node and image slot (0:
/// the content image).
#[derive(Default)]
pub(crate) struct Playback(pub(super) BTreeMap<(NodeId, u8), Entry>);

/// One playing image: the asset it plays, its timing, and where it is.
pub(super) struct Entry {
    /// The asset the entry plays (a change starts the entry again).
    pub(super) asset: Asset,
    /// The animation of the asset: each frame's delay and the repeat.
    pub(super) info: AnimationInfo,
    /// Where the playback is.
    pub(super) state: PlaybackState,
}

/// Where a playback is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaybackState {
    /// Playing since `start` (host ns).
    Playing { start: u64 },
    /// The repeat count is spent: the last frame is shown.
    Finished,
}

/// The length of one cycle of `info` in ns (at least one ns).
fn cycle_ns(info: &AnimationInfo) -> u64 {
    let ms: u64 = info.delays_ms.iter().map(|d| u64::from(*d)).sum();
    ms.max(1).saturating_mul(1_000_000)
}

/// The end of each frame within a cycle, in ms, in frame order.
fn frame_ends(info: &AnimationInfo) -> impl Iterator<Item = u64> + '_ {
    info.delays_ms.iter().scan(0u64, |end, delay| {
        *end = end.saturating_add(u64::from(*delay));
        Some(*end)
    })
}

/// The host time (ns) a run from `start` ends at; `None` when it repeats
/// forever.
fn run_end(info: &AnimationInfo, start: u64) -> Option<u64> {
    match info.repeat {
        Repeat::Times(n) => Some(start.saturating_add(u64::from(n).saturating_mul(cycle_ns(info)))),
        Repeat::Forever => None,
    }
}

/// The index of the last frame of `info`.
fn last_frame(info: &AnimationInfo) -> u32 {
    u32::try_from(info.delays_ms.len().saturating_sub(1)).unwrap_or(u32::MAX)
}

/// The frame `entry` shows at host time `now`.
fn frame_at(entry: &Entry, now: u64) -> u32 {
    let info = &entry.info;
    let PlaybackState::Playing { start } = entry.state else {
        return last_frame(info);
    };
    if run_end(info, start).is_some_and(|end| now >= end) {
        return last_frame(info);
    }
    let offset_ms = now.saturating_sub(start) % cycle_ns(info) / 1_000_000;
    match frame_ends(info).position(|end| end > offset_ms) {
        Some(index) => u32::try_from(index).unwrap_or(last_frame(info)),
        None => last_frame(info),
    }
}

/// The first host time after `now` at which the frame of `entry` changes,
/// within its run (`None` when it is not playing or its run has ended).
fn next_switch(entry: &Entry, now: u64) -> Option<u64> {
    let PlaybackState::Playing { start } = entry.state else {
        return None;
    };
    let info = &entry.info;
    let period = cycle_ns(info);
    let elapsed = now.saturating_sub(start);
    let cycle_start = elapsed - elapsed % period;
    let offset_ms = elapsed % period / 1_000_000;
    let end_ms = frame_ends(info).find(|end| *end > offset_ms);
    let next = start
        .saturating_add(cycle_start)
        .saturating_add(end_ms.map_or(period, |end| end.saturating_mul(1_000_000)));
    match run_end(info, start) {
        Some(end) => (end > now).then_some(next.min(end)),
        None => Some(next),
    }
}

/// Finish `entry` when its run has ended at `now`.
fn settle(entry: &mut Entry, now: u64) {
    if let PlaybackState::Playing { start } = entry.state
        && run_end(&entry.info, start).is_some_and(|end| now >= end)
    {
        entry.state = PlaybackState::Finished;
    }
}

impl Model {
    /// Bring the playbacks up to date with the images drawn now: an entry
    /// is dropped when its image no longer plays or its asset changed, and a
    /// new animated image starts at this frame.
    pub(crate) fn sync_playback(&mut self) {
        let wanted = self.playing_images();
        self.playback
            .0
            .retain(|(id, _), entry| wanted.get(id) == Some(&entry.asset));
        let now = self.frame.now();
        for (id, asset) in wanted {
            if self.playback.0.contains_key(&(id, 0)) {
                continue;
            }
            let Some(info) = self.assets.borrow_mut().animation(&asset) else {
                continue;
            };
            let mut entry = Entry {
                asset,
                info,
                state: PlaybackState::Playing { start: now },
            };
            settle(&mut entry, now);
            if let Some(at) = next_switch(&entry, now) {
                self.frame.request(at);
            }
            self.playback.0.insert((id, 0), entry);
        }
    }

    /// The content images drawn now that play: their `playing` prop is not
    /// false, and reduced motion is off.
    fn playing_images(&self) -> BTreeMap<NodeId, Asset> {
        if self.frame.reduced_motion() {
            return BTreeMap::new();
        }
        self.tree
            .iter()
            .filter_map(|(id, node)| {
                let asset = node.image()?.clone();
                let playing = node.image_data().and_then(|d| d.playing).unwrap_or(true);
                (playing && self.tree.drawn(id)).then_some((id, asset))
            })
            .collect()
    }

    /// The frame of the content image of `id` to draw now (0 without a
    /// playback).
    pub(crate) fn playback_frame(&self, id: NodeId) -> u32 {
        self.playback
            .0
            .get(&(id, 0))
            .map_or(0, |entry| frame_at(entry, self.frame.now()))
    }

    /// At a tick at `now`: finish the runs that ended, repaint the playing
    /// images (the tick is due at a frame change) and ask for their next.
    pub(crate) fn step_playback(&mut self, now: u64) {
        let mut repaint = Vec::new();
        for (&(id, _), entry) in self.playback.0.iter_mut() {
            let was_playing = matches!(entry.state, PlaybackState::Playing { .. });
            settle(entry, now);
            if !was_playing {
                continue;
            }
            repaint.push(id);
            if let Some(at) = next_switch(entry, now) {
                self.frame.request(at);
            }
        }
        for id in repaint {
            self.repaint(id);
        }
    }
}
