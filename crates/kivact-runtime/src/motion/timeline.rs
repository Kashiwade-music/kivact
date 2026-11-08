//! Role: `Timeline`: tracks that move along one clock, with labels, calls,
//! repeats and playback controls.

mod api;
mod calls;
mod curve;
mod plan;
mod segment;

pub use api::{At, Segment, Timeline, TimelineTrack, timeline};
