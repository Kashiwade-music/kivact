//! Role: Enter and exit paint: where a node starts at mount and ends before it goes.
//!
//! A node's enter paint (`StateStyles::enter`, CSS `@starting-style`) is
//! where its paint transition starts when it is first attached; its exit
//! paint (`StateStyles::exit`) is where the transition takes a removed
//! node before it goes. Both are state deltas over the others (`states`)
//! moved by the node's paint transition (`state_motion`), so only what
//! that transition interpolates enters and leaves; no animation of its
//! own exists for them. Kivact's removal stays one write: the engine keeps
//! what has an exit to show.
//!
//! Invariants:
//! - A created node with an enter paint is captured at it once, at the
//!   first commit that finds it attached, and the next retarget moves it
//!   to its paint; one without an enter paint at the first commit after
//!   its creation (an enter added later) never enters.
//! - A removed subtree that has an exit to show stays in the tree as
//!   leaving (every node of it): it is drawn where it was laid out, but it
//!   is never hit, focused, exposed, observed or in a modal scope, and the
//!   focus inside it moves out at once, as for a removal; a scrollbar or
//!   file drag on it ends. Kivact forgot it at the removal and never names
//!   it again. A subtree that is not drawn (detached, or hidden) goes at
//!   once.
//! - A leaving subtree is removed at the first sync after no node in it
//!   moves. By default it keeps its place in the layout until then. A
//!   removal can instead pin the last local box outside flow so siblings
//!   move immediately. One whose ancestor kivact removes goes with the
//!   ancestor. A requested completion is reported on physical removal.
//! - Without a paint transition, or under reduced motion with a decorative
//!   one, a node neither enters nor leaves: it shows its paint and goes at
//!   once.

mod size;
pub(crate) mod state;
mod transitions;
