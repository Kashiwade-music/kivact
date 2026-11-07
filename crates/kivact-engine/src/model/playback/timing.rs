//! Role: Pure timing of a playback: its cycle, its frame at a time and its next change.
//!
//! Every function is a function of the entry and the host time, so the paint
//! and the ticks agree on the frame without sharing a store.

use arvane::asset::AnimationInfo;
use kivact_types::motion::Repeat;

use super::state::{Entry, PlaybackState};

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
pub(super) fn frame_at(entry: &Entry, now: u64) -> u32 {
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
pub(super) fn next_switch(entry: &Entry, now: u64) -> Option<u64> {
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
pub(super) fn settle(entry: &mut Entry, now: u64) {
    if let PlaybackState::Playing { start } = entry.state
        && run_end(&entry.info, start).is_some_and(|end| now >= end)
    {
        entry.state = PlaybackState::Finished;
    }
}
