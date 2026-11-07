//! Role: Export public animation values and node-aware frame loops.

mod frame_loop;

pub use frame_loop::{FrameLoop, frame_loop};
#[doc(inline)]
pub use kivact_runtime::{
    Animatable, Animated, At, Easing, FrameInfo, FrameRate, Repeat, Retarget, Segment, Spring,
    Stagger, StaggerFrom, StepPosition, Timeline, TimelineTrack, Transition, animated, timeline,
};
