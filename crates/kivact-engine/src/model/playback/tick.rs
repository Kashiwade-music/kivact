//! Role: The model's steps for the animated image playbacks: sync, frame, tick.

use std::collections::BTreeMap;

use kivact_types::paint::{Fill, FillImage};
use kivact_types::{Asset, NodeId};

use super::state::{Entry, PlaybackState, background_slot};
use super::timing::{frame_at, next_switch, settle};
use crate::model::Model;

/// The playing images, by node and slot: the content image of each drawn node
/// (slot 0), and each animated background layer of a drawn node (its slot).
type Wanted = BTreeMap<(NodeId, u8), Asset>;

impl Model {
    /// Bring the playbacks up to date with the images drawn now: an entry
    /// is dropped when its image no longer plays or its asset changed, and a
    /// new animated image starts at this frame.
    pub(crate) fn sync_playback(&mut self) {
        let wanted = self.playing_images();
        self.playback
            .0
            .retain(|key, entry| wanted.get(key) == Some(&entry.asset));
        let now = self.frame.now();
        for (key, asset) in wanted {
            if self.playback.0.contains_key(&key) {
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
            self.playback.0.insert(key, entry);
        }
    }

    /// The animated images drawn now that play: a content image whose
    /// `playing` prop is not false, and each animated background layer. Reduced
    /// motion plays none.
    fn playing_images(&self) -> Wanted {
        let mut wanted = Wanted::new();
        if self.frame.reduced_motion() {
            return wanted;
        }
        for (id, node) in self.tree.iter() {
            if !self.tree.drawn(id) {
                continue;
            }
            if let Some(asset) = node.image() {
                let playing = node.image_data().and_then(|d| d.playing).unwrap_or(true);
                if playing {
                    wanted.insert((id, 0), asset.clone());
                }
            }
            let background = node.paint().and_then(|p| p.background.as_ref());
            if let Some(Fill::Layers(layers)) = background {
                for (index, layer) in layers.iter().enumerate() {
                    if let FillImage::Image(asset) = &layer.image {
                        wanted.insert((id, background_slot(index)), asset.clone());
                    }
                }
            }
        }
        wanted
    }

    /// The frame of the content image of `id` to draw now (0 without a
    /// playback).
    pub(crate) fn playback_frame(&self, id: NodeId) -> u32 {
        self.playback_frame_at(id, 0)
    }

    /// The frame of playback `slot` of node `id` to draw now (0 without one).
    pub(crate) fn playback_frame_at(&self, id: NodeId, slot: u8) -> u32 {
        self.playback
            .0
            .get(&(id, slot))
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
