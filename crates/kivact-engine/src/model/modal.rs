//! Role: Modal overlays that confine input, Tab order and focus to their subtree.
//!
//! Modal overlays (`OverlayProps::modal`): while one is shown, input, Tab
//! order and focus are confined to the subtree of the topmost shown modal
//! overlay (the last one in tree order). This module is the one authority
//! for admitting input to a node, for whether a focus restore may land and
//! for the overlay an unhandled Escape dismisses; kivact asks it through
//! `Reader` and keeps no modal index of its own.
//!
//! Invariants:
//! - Only the overlay list is scanned, so the lookups cost nothing while no
//!   modal overlay exists; the tree is walked only when several are shown.
//! - A node is in scope when no modal is shown or it lies in the subtree of
//!   the active modal; overlays nested inside the modal are in scope. A
//!   hidden modal confines nothing.
//! - Ancestor searches (pointer, wheel, focus and activation targets) and
//!   scroll chains that start inside the active modal end at it.
//! - At every commit, a focus outside the active modal moves to the next
//!   tab stop inside it (in tree order after the old focus, wrapping), or
//!   to nothing when the modal has no tab stop.
//! - Admission ([`Model::admit_input`]) is decided once per event, when
//!   kivact is about to dispatch it, so a modal an earlier event of the
//!   same turn opened already applies. A node event (`Activate`, `A11y`,
//!   `Wheel`, `File`, `TextChanged`, `SelectionChanged`, `OverlayDismiss`)
//!   is admitted exactly when its node is in scope. A pointer event that
//!   starts something (a `Down`, an `Enter`, a `Move` with no button held)
//!   needs scope; the later `Move`, `Up` and `Cancel` of a gesture
//!   admitted at its `Down` stay admitted on that node whatever modal
//!   opens meanwhile, and the final release or the `Cancel` ends it. A
//!   rejected event for another node does not end an admitted drag, and
//!   `Leave` is always admitted. Events not aimed at a node pass. The
//!   drag table lives in [`state::Admission`] and is borrowed inside one
//!   call only.
//! - A leaving node is in no modal scope, and a leaving overlay neither
//!   confines nor dismisses (`NodeTree::shown`).

mod confinement;
pub(crate) mod state;
