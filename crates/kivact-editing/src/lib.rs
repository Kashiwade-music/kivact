//! Role: Typed editing state and text layout contracts.
//!
//! Owned native editing state, composition and shaped layout snapshots.
//! An [`EditState`] is a document (the text and its revision) with an
//! editing session over it ([`TextSession`]: the selection's anchor and
//! caret, and the IME composition). Every change of the text is a range
//! replacement ([`TextChange`]), recorded as it is made, so the host
//! reports edits by range instead of by whole value. An editor runs
//! kivact's hooks: its behavior ([`EditBehavior`]) for each routed key and
//! paste, given the editor's constraints and laid-out lines
//! ([`EditContext`]), and its line colors ([`LineColors`]), one hard line
//! at a time from the state the line before left ([`LineState`]).
//!
//! Invariants:
//! - Offsets are byte offsets into the text on char boundaries; platform
//!   IMEs see UTF-16 ones ([`EditState::utf16_of_byte`]), converted at the
//!   edges.
//! - A selection is its anchor and its caret, in either order; the selected
//!   range runs from the smaller to the larger.
//! - Every change of the text takes a new revision from one counter, so
//!   equal revisions mean equal text; comparing states compares their
//!   texts only when the revisions differ.
//! - The text changes only through recorded range replacements, kept with
//!   the text they removed (so the host can revert them), in order.
//! - Moving the caret or the selection never changes the text: the host sends
//!   `TextChanged` only when the text changed.
//! - A value set by the core waits while a composition is in progress; the
//!   host gives it again when the composition ends, as a recorded change
//!   (`Journal::adopt`).
//! - A host changes an editor's text by one [`Journal`] per edit: it opens on
//!   the current text and session, ends with the changes to report, or takes
//!   them all back (also when dropped unfinished).
//! - The caret starts at the end of the value.
//! - Every change of the text is kept in a short record by revision, so each
//!   line cache asks what changed since its own revision
//!   (`EditState::damage_since`).
//! - An [`EditContext`] answers for the text the request found: once the
//!   behavior changed the state's text (a new revision), its layout
//!   queries answer `None`.
//! - Two behaviors, or two line colors, are equal when they are the same
//!   function object. A line's colors are clamped to the line's char
//!   boundaries, sorted and cut where they overlap.

#![forbid(unsafe_code)]

mod behavior;
mod change;
mod edit_state;
mod editor;
mod geometry;
mod journal;
mod line_colors;
mod session;
mod text_layout;

#[doc(hidden)]
/// Internal contract: the native host (kivact-engine) drives the editor through these.
pub mod host;

pub use behavior::{EditBehavior, EditContext, EditLayout, EditOutcome, EditRequest, Lines};
pub use change::TextChange;
pub use edit_state::EditState;
pub use geometry::{TextGeometry, caret_at};
pub use line_colors::{LineColors, LineState, LineStyle, RangeDecoration, RangeDecorationKind};
pub use session::TextSession;
pub use text_layout::{EditCaret, EditLine, TextLayout};
