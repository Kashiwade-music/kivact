//! Role: Public animation names of the facade: frame loops, values and transitions.
//!
//! The implementation is the `kivact-motion` crate; this module re-exports the
//! names an app or a component author calls (`frame_loop`, `animated`,
//! `Transition`, ...). The driver API of the host that runs the App's frames
//! (`MotionState`, `MotionStats`) stays in `kivact_motion` and is not part of
//! the public facade.

#[doc(inline)]
pub use kivact_motion::{
    Animatable, Animated, Easing, FrameInfo, FrameLoop, FrameRate, Repeat, Spring, Transition,
    animated, frame_loop,
};
