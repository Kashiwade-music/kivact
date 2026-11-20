//! Role: Virtualized list with viewport-based row mounting.
//!
//! `virtual_list`: a scroll box whose visible range is computed by the core
//! from `ScrollChanged` (offset, viewport). Only visible rows (plus overscan)
//! are mounted; each row is an absolutely positioned wrapper. The host needs
//! no virtualization-specific node kind. Each row is set up in its own
//! owner; the row builder receives the list source's row handle
//! (`ListSource::Row`), as `for_each` does: a `Value<T>`, or a `Signal<T>`
//! that writes back for a `Signal` or `Signal` source.
//!
//! The list is a scroll box, spacers and absolutely positioned rows; core
//! computes the visible range (±200 px of overscan by default).
//!
//! Invariants:
//! - `scroll_to(Option<key>)` acts each time the value becomes `Some`: it
//!   computes the row's position from the row heights (`Fixed`, or the
//!   measured heights and the estimate), moves the list with
//!   `Writer::scroll_to`, and finishes with `ScrollIntoView` once the row
//!   is mounted, at most 8 steps per request. `List::scroll_with` hands the
//!   target key to `scroll_to`, and a row that is not mounted takes the
//!   focus when it mounts.
//! - `keep_focused` (default `true`) keeps the row holding the focus (found
//!   by walking up from the focused node to a child of the spacer) mounted
//!   at its absolute position outside the range. An effect that reads only
//!   the focus source decides that row, and the range is computed again
//!   only when the row changes.
//! - Mounting 100 rows or more under an unconstrained size is warned
//!   about.

mod builder;
mod heights;
mod mount;
mod rows;
mod seeker;
mod state;

pub use builder::{RowHeight, VirtualList, virtual_list};
