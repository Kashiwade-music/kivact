//! Role: Player state shared through context.
//! The rodio player is neither `Clone`
//! nor `PartialEq`; it lives in a `Signal` and is changed with `update`
//! (always notifies). Readers derive small memos that stop on equal values.

use crate::audio::{AudioPlayer, SharedClip};
use kivact::prelude::*;
use std::time::Duration;

#[derive(Clone, Copy)]
pub(crate) struct PlayerCx {
    pub player: Signal<AudioPlayer>,
    /// Loads a WAV path (`Some`) or the generated demo (`None`) on a worker.
    pub loader: Action<SharedClip, Option<String>>,
    pub wav_path: Signal<String>,
    /// Identity of the loaded clip (waveform redraws only when it changes).
    pub clip: Value<SharedClip>,
    pub position: Value<Duration>,
    pub playing: Value<bool>,
}

impl PlayerCx {
    pub(crate) fn seek_to(self, position: Duration) {
        self.player.update(|p| p.seek(position));
    }

    pub(crate) fn seek_by(self, seconds: i64) {
        self.player.update(|p| p.seek_delta(seconds));
    }
}
