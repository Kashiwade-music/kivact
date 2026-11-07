//! Role: `Timeline`: tracks that move along one clock, with labels, calls,
//! repeats and playback controls (design §12.6).

mod api;
mod curve;
mod plan;
mod segment;

pub use api::{At, Segment, Timeline, TimelineTrack, timeline};
