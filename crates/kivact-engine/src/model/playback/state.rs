//! Role: Passive state of the animated image playbacks, keyed by node and slot.
//!
//! The model's aggregate holds these types by their path; they name no
//! model type, so the aggregate and the playback steps need no cycle.

use std::collections::BTreeMap;

use arvane::asset::AnimationInfo;
use kivact_types::paint::{Fill, FillImage};
use kivact_types::{Asset, NodeId};

use crate::tree::node::HostNode;

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

/// The playback slot of background layer `index` (slot 0 is the content image).
pub(crate) fn background_slot(index: usize) -> u8 {
    u8::try_from(index + 1).unwrap_or(u8::MAX)
}

/// The images of `node` that play when it is drawn: its content image
/// (slot 0) unless its `playing` prop is false, then each animated background
/// layer in its slot. Reduced motion is applied by the reader.
pub(crate) fn node_images(node: &HostNode) -> Vec<(u8, Asset)> {
    let mut images = Vec::new();
    if let Some(asset) = node.image() {
        let playing = node.image_data().and_then(|d| d.playing).unwrap_or(true);
        if playing {
            images.push((0, asset.clone()));
        }
    }
    if let Some(Fill::Layers(layers)) = node.paint().and_then(|p| p.background.as_ref()) {
        for (index, layer) in layers.iter().enumerate() {
            if let FillImage::Image(asset) = &layer.image {
                images.push((background_slot(index), asset.clone()));
            }
        }
    }
    images
}
