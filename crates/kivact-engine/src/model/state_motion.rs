//! Role: Transitions of state styles; only transitioning nodes allocate them.
//!
//! A node's paint transitions (`PaintProps::transitions`, one per
//! `TransitionProp`, `All` for the properties without their own) move its
//! paint from what is shown to its target paint, the same way whatever
//! changed the target: a paint or state style write (a signal, a style
//! change), the host's hover, pressed or focus, a group state, an
//! entrance or an exit. Each property has a track (`PropTrack`) that moves
//! the numbers `interp::prop_interp` takes from the paint and the
//! text color: the canonical transform's parts and origin component by
//! component (so `rotate(0)` to `rotate(360)` turns once); what it does
//! not interpolate switches to the target at once.
//!
//! Invariants:
//! - A state change captures the displayed value first
//!   (`Model::prepare_state_motion`, `TrackPhase::Captured`); sync retargets
//!   the captures once (`TrackPhase::Running`); frames step only running
//!   ones. A run that ends away from the target (alternate, even count)
//!   is held (`TrackPhase::Held`) until the next capture.
//! - Under reduced motion a decorative transition ends at once; a
//!   transition whose target is what is shown ends without a frame.
//! - A moving transform asks for a layout pass every frame (the node is
//!   placed, hit and reported where it is drawn).
//! - While no window can draw, the App clock stands still (kivact_types::clock::Clock), so the transitions go on from where they stood.

pub(crate) mod interp;
pub(crate) mod state;
mod steps;
