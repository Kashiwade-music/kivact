//! Role: Which frame each drawn animated image shows, and when it changes.
//!
//! The content image of a node plays when it is drawn, its `playing` prop
//! is not false, reduced motion is off and its asset is animated. It plays
//! from the frame it was first drawn in, repeats its run (`Repeat`), and
//! then holds its last frame. An animated background layer plays the same
//! way (in its own slot).
//!
//! Invariants:
//! - An entry exists only for a drawn image that plays; it is dropped when
//!   the image stops playing or its asset changes (then it starts again).
//! - Frame changes are ticks: each playing image asks for its next change
//!   and is repainted at it (`Model::step_playback`). Nothing is requested
//!   for a finished run.
//! - A frame is a pure function of the run's start and the time, so the
//!   paint never needs the store (`Model::playback_frame`).

pub(crate) mod state;
mod tick;
mod timing;
