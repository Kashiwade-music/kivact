//! Role: The model's steps for the animated image playbacks: sync, frame, tick.

use std::collections::BTreeMap;

use kivact_types::{Asset, NodeId};

use super::state::{Entry, PlaybackState};
use super::timing::{frame_at, next_switch, settle};
use crate::model::Model;

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
