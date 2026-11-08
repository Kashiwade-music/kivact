//! Role: Animation system with frame-driven updates.
//!
//! Animation, one system: a single host-driven clock, `FrameLoop`
//! callbacks and `Animated` values that are graph sources.
//!
//! Invariants:
//! - There is one clock. Each flush computes the earliest deadline of what
//!   moves by frames and writes one `Writer::request_frame(at_ns)` per
//!   request; there is no fixed timer.
//! - While nothing can be drawn (`Environment::surface_active` is false: no
//!   surface, an empty window, an App suspended to the background) no
//!   frame is requested and the frame clock stops. The engine owns that
//!   clock (`MotionClock`, carried by every `FrameTick`): when drawing
//!   resumes, the first tick says so, and a frame loop counts from that
//!   tick (no burst of ticks) while an animation goes on from the time it
//!   stopped, so a transition does not finish while the window is hidden.
//!   Reduced motion is applied in the turn that reports it, not at a tick.
//! - `FrameRate::Hz(r)` puts the k-th tick at `start + round(k * 1e9 / r)`.
//! - An [`Animated`] whose target changes heads for it from its current
//!   value (and, for a spring, its velocity). [`Animated::from`] moves from
//!   its start on the first mount too. A retarget while paused keeps the
//!   current value and heads for the new target after `resume`.
//! - `on_complete` (registered in setup) reports a natural completion and
//!   one that reduced motion forces: once for a whole finite repeat, never
//!   for an infinite repeat, a superseded target, a target equal to the
//!   current value (no run starts), a `jump_to` or a disposed owner. It runs
//!   as an effect after the interpolation and the registry borrow ended.
//! - Reduced motion finishes decorative transitions at once.
//! - [`Transition::repeat`] ([`Repeat`]) and [`Transition::alternate`]
//!   (every other iteration backwards) write the repeat; the engine's
//!   `PlayDirection` is internal to its wire and not re-exported. One
//!   stepper (`kivact_types::motion::Transition::step`) plays client
//!   animations and the host's state transitions. `Animated` uses the wire
//!   timing the bridge re-exports, and the public `Easing`, `Spring` and
//!   `Repeat` are those single definitions, the ones the engine uses for
//!   style paint and for the host's focus, hover and pressed transitions
//!   (one timing, spring integration and paint interpolation).
//!
//! Entering and leaving ride on the state styles and the paint transition;
//! there is no animation type for them. `Style::enter` (CSS
//! `@starting-style`, Framer Motion's `initial`) is the paint a node starts
//! from when it is first attached, and `Style::exit` the paint it moves to
//! when it is removed; both are `StateStyles` deltas (`enter`, `exit`)
//! moved by the node's `TransitionProp::All` transition (only the
//! properties that transition interpolates move). The engine keeps a
//! removed subtree with an exit as leaving (drawn, but out of hit testing,
//! focus, accessibility, modal scope and layout observation) until it
//! settles; core forgets the nodes at the removal and never names them
//! again. A node without a paint transition, or with a decorative one under
//! reduced motion, enters and leaves at once. The standard toast, dialog,
//! popup and tooltip use this through the styles their recipes return
//! (Game's toast slides in with `enter(style().translate_x(40).opacity(0.))`);
//! a dialog has no entering value of its own.
//!
//! The paint transitions of styles and of the host's states run in the
//! engine (`kivact-engine`): a style writes only the target, the client
//! writes no paint per frame, and only moving nodes are stepped. Their
//! interpolated transform is what placement, hits, rects, accessibility and
//! IME read.

mod animated;
mod frame_loop;
mod stagger;
mod state;
mod timeline;
mod transition;
mod values;

pub use animated::{Animated, animated};
pub use frame_loop::{FrameLoop, frame_loop};
pub use kivact_types::motion::{Easing, Keyframe, Repeat, Retarget, Spring, StepPosition};
pub use stagger::{Stagger, StaggerFrom};
pub use state::{AnimInfo, AnimKind, MotionEnv, MotionState, MotionStats};
pub use timeline::{At, Segment, Timeline, TimelineTrack, timeline};
pub use transition::Transition;
pub use values::{Animatable, FrameInfo, FrameRate};
