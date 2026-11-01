//! Role: A text input's lines, shaped as they are needed and kept in step with its edits.
//!
//! Refresh before geometry queries and keep the displayed source consistent
//! with that refresh. Ordinary reconciliation is incremental. Explicit
//! [`RefreshMode::Verify`] checkpoints add a complete metadata scan only in
//! debug builds; release compiles that assurance and its allocations out.
//! No query owns or mutates an editor document or its change history.
//!
//! [`EditLines`]: the hard lines of what a text input shows (its value,
//! its masked value, or its placeholder while the value is empty), each
//! shaped as a paragraph of its own when something first needs it (it is
//! drawn, measured, under the caret or the pointer, or published), with
//! what drawing and assistive technology make of each line (its colors,
//! its colored glyphs, its text run). An edit splits again only the lines
//! its damage touches (`Editor::take_damage`); the lines after them
//! move with their chunk. So opening a document costs splitting and
//! counting its text, and an edit, the caret, the pointer, the IME and the
//! accessibility runs cost the lines they touch and the chunk list, not
//! the document's text.
//!
//! The editor measures its natural width from the lines that start in its
//! first 16 KiB ([`EditLines::measured_width`]), not from every line, so a
//! wide line late in a long document is not part of a content-sized
//! editor's width (`width: fit-content` too): it scrolls across into view
//! (the caller chooses scrolling geometry). The caret, the pointer, the selection and the IME's
//! character boxes read their own lines; drawing shapes only the lines
//! that reach the clip and the content box the paint list gives the node
//! ([`EditLines::rows`]), and colors them only while they show an unmasked
//! value. The IME is given the lines around the caret while they fit in
//! 2048 bytes on each side, else the part of its own lines that
//! does ([`EditLines::excerpt`]).
//!
//! Shaping a line at a time is what made long documents work: parley keeps
//! a cluster's text offset as a `u16` from its run's start, so one
//! paragraph past 64 KiB got wrong line ranges and carets, and its
//! accessibility runs stopped being contiguous, so the platform refused the
//! whole tree (a 10,000-line editor could not be found by its role). Cost,
//! measured in a release build on a real Windows window with the GPU and
//! `KIVACT_TEST_LIVE=1` (a frame per input): a plain editor
//! (`text_edit_policy`, lines of about 30 bytes) in a 420 × 220 scroll,
//! scrolled to its middle line, the median of 12 one-character inserts at
//! its start until the next frame is drawn, and opening from the mount to
//! the first frame. Before, with one paragraph: opening 22 ms for 1,000
//! lines, 181 ms for 10,000, 2.01 s for 100,000 and 19.2 s for 1,000,000;
//! a character 2.6, 3.2, 6.8 and 49 ms. After: opening 15, 17, 48 and
//! 88 ms; a character 2.7, 2.8, 2.6 and 4.0 ms.
//!
//! What still grows with the document: splitting it and counting each
//! line's scalars and UTF-16 units on opening (tens of ms for a million
//! lines), the memmove of an insert into the one `String` an `EditState`
//! keeps (most of the million-line insert), recounting the chunk list (a
//! 256th of the lines), and coloring the lines in order up to the drawn
//! ones.
//!
//! Invariants:
//! - A multiline input's lines end after each line feed, which belongs,
//!   with a CR before it, to the line it ends; the last line has no
//!   break (it is empty after a final line feed). A single-line input is
//!   one line, its line feeds shown as spaces. A lone CR shows as a space
//!   (Arvane would break there; editing knows only line feeds). Masked
//!   text shows one bullet per scalar, so carets keep their offsets.
//! - Lines are shaped unwrapped, start aligned and without a color, and
//!   lie `height` apart (the shaper's line height), shaped or not: a line
//!   shaped late moves no other line. The input's alignment moves each
//!   line by an offset in its box ([`EditLines::offset`]), and its color
//!   is the drawing's. A line's carets are indexed by scalar offset in
//!   the line. A line longer than 16 KiB is shaped in pieces (`pieces`)
//!   cut at grapheme boundaries (after white space where it can), joined
//!   end to end into one shaped line.
//! - The lines are kept in chunks of 256 lines (256), at most twice
//!   as many: an edit cuts a longer chunk, and joins one left under a
//!   quarter of `CHUNK` to the next when that one is not over `CHUNK`. A
//!   line's offsets are from its chunk's start, so an edit moves the lines
//!   after it only within its chunk, and `before` holds, for every chunk,
//!   the lines, bytes, scalars and UTF-16 units before it, counted again
//!   from the edited chunk on.
//! - The lines are of the text at `revision`. When the text moved on by a
//!   damage that starts from that revision, the lines from the one
//!   holding the damage's start to the one holding its old end are split
//!   again (to be shaped when needed); anything else (another style, what
//!   is shown or another placeholder, a state replaced without damage, a
//!   damage that does not fit) splits every line again, and a registered
//!   font drops the lines (clients invalidate after font registration: fallback may change).
//! - Line colors are taken in line order: the lines before `colored` were
//!   colored from the state the line before them left. A line keeps its
//!   colors while its text and the state it was colored from stay the
//!   same, so an edit colors again its own lines, and the lines after it
//!   only as far as the state they start from changed and as far as they
//!   are drawn.
//! - A line's text run (accessibility) and its colored glyphs are kept
//!   with the line until it changes, its colors change or (the run) it
//!   moves. Only the lines around the caret and the lines last drawn are
//!   published ([`EditLines::published`]): the caret's line and, while the
//!   caret is within 32 lines of them, the lines last
//!   drawn, with that many more on each side, widened to multiples of
//!   32 (32), so a scroll replaces the runs only when it
//!   crosses a step, as a browser's editor gives assistive technology the
//!   text around its caret.

#![forbid(unsafe_code)]

mod chunks;
mod edit_lines;
mod excerpt;
#[cfg(debug_assertions)]
mod integrity;
mod line_metrics;
mod pieces;
mod records;
mod refresh;
mod shaping;

pub use edit_lines::EditLines;
pub use line_metrics::{line_height, line_offset};
pub use refresh::{RefreshError, RefreshMode, Source};
pub use shaping::{Published, Shaping};
