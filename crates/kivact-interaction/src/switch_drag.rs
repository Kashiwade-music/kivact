//! Role: Opt-in direct manipulation of a switch thumb.
mod api;
mod machine;
mod types;

pub use api::{SwitchDragApi, use_switch_drag};
pub use types::{Lift, SwitchTrack, ThumbDrag, TrackInput};
