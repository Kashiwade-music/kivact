//! Role: A playable Kivact tutorial: shared PCM drives audio, waveform and spectrum.
//! The rodio player is a non-Clone object held in a `Signal`; a frame loop
//! republishes its position while it plays.
#![warn(redundant_imports)]

mod analysis;
mod audio;
mod components;
mod player_state;
mod waveform_index;

pub use audio::{AudioClip, format_time};
pub use components::player;
