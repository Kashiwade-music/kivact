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

use kivact_editing::{Damage, LineColors};
use records::{Before, Chunk, Key};
use std::ops::Range;
use std::sync::Arc;

mod chunks;
mod excerpt;
#[cfg(debug_assertions)]
mod integrity;
mod pieces;
mod records;
mod refresh;
mod shaping;

/// Line height of `style` as Arvane's shaper lays lines out: the height of
/// the caret of an empty text (the font size if `style` cannot be shaped).
pub fn line_height(engine: &mut arvane::Engine, style: &ui_types::TextStyle) -> f32 {
    engine
        .shape("", style, None)
        .ok()
        .and_then(|s| s.carets.first().map(|c| c.rect.height))
        .unwrap_or(style.font_size)
}

/// How far below the top of a content box `height` high a single-line
/// input's line of `line` height starts: the line is centered, and a box
/// lower than the line (negative `height - line`) overflows it evenly on
/// both sides. Callers can share this value for content-box placement and
/// measured baseline positioning.
pub fn line_offset(height: f32, line: f32) -> f32 {
    (height - line) / 2.0
}

/// What a text input shows, to lay its lines out from.
#[derive(Clone, Copy)]
pub struct Source<'a> {
    /// Borrowed original UTF-8 value; all source byte offsets refer to it.
    pub value: &'a str,
    /// Revision of the value and compatible damage history.
    pub revision: u64,
    /// Optional UTF-8 byte replacement since the previously refreshed revision.
    pub damage: Option<Damage>,
    /// Borrowed UTF-8 display text used only while the value is empty.
    pub placeholder: &'a str,
    /// Visually mask non-LF scalars with bullets; LF keeps display spacing.
    /// This does not redact excerpts or published raw run text. Callers must
    /// suppress/protect surrounding IME text and accessible publication.
    pub password: bool,
    /// Split at LF hard breaks; an unmasked single line shows LF/CR as spaces.
    pub multiline: bool,
}

/// Cost and diagnostic assurance requested for one source reconciliation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RefreshMode {
    /// Reconcile incrementally without a complete representation scan.
    #[default]
    Incremental,
    /// In debug builds, independently verify retained line metadata afterward.
    /// This expensive checkpoint is useful after a source batch or adoption.
    /// In release builds it performs ordinary reconciliation only; no integrity
    /// scan, reference allocation or diagnostic assurance is provided.
    Verify,
}

/// Discrepancies found at an explicitly requested debug integrity checkpoint.
/// The source and retained cache are left intact for diagnosis, never healed
/// from the independent reference. No private representation is exposed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshError {
    /// Located descriptions of retained/reference metadata differences.
    pub discrepancies: Vec<String>,
}

impl std::fmt::Display for RefreshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.discrepancies.join("; "))
    }
}
impl std::error::Error for RefreshError {}

/// Bytes of text on each side of the caret the IME is given, at most
/// (the lines around it while they fit).
const IME_CONTEXT: usize = 2048;

/// Lines per chunk: a chunk holds at most twice as many.
const CHUNK: usize = 256;

/// The text measured for an input's natural width: the lines starting in
/// its first this many bytes (at least its first line).
const MEASURED: usize = 16 * 1024;

/// Lines published around the caret and the lines drawn, on each side,
/// and the step the published lines start and end at.
const PUBLISHED_MARGIN: usize = 32;
const PUBLISHED_STEP: usize = 32;

/// The lines of a text input (see the module invariants).
#[derive(Clone, Debug)]
pub struct EditLines {
    key: Option<Key>,
    align: ui_types::TextAlign,
    revision: u64,
    height: f32,
    /// The baseline of a line without text (a space's, in the style).
    blank: f32,
    chunks: Vec<Chunk>,
    /// What comes before each chunk, then in all.
    before: Vec<Before>,
    /// The widest shaped line's width.
    width: f32,
    /// The colors the lines were colored with.
    colors: Option<LineColors>,
    colored: usize,
    /// The lines last drawn.
    drawn: Range<usize>,
}

impl Default for EditLines {
    /// No lines yet, with what comes before them all (nothing): `before`
    /// always ends with the totals.
    fn default() -> Self {
        Self {
            key: None,
            align: ui_types::TextAlign::default(),
            revision: 0,
            height: 0.0,
            blank: 0.0,
            chunks: Vec::new(),
            before: vec![Before::default()],
            width: 0.0,
            colors: None,
            colored: 0,
            drawn: 0..0,
        }
    }
}

/// The text runs published of a text input's lines: from scalar `start`
/// of its text to scalar `end`.
pub struct Published {
    /// First Unicode scalar covered by the returned runs.
    pub start: u32,
    /// Exclusive final Unicode scalar covered by the returned runs.
    pub end: u32,
    /// Owned shared runs, retaining their font-independent geometry and values.
    /// Allocation identity is not a publication equality contract.
    pub runs: Arc<[Arc<ui_types::SemanticsTextRun>]>,
}

/// An input's lines as a behavior reads them, shaping the lines it asks
/// about.
pub struct Shaping<'a> {
    lines: std::cell::RefCell<(&'a mut EditLines, &'a mut arvane::Engine)>,
    revision: u64,
}

impl EditLines {
    fn key(&self) -> &Key {
        self.key
            .as_ref()
            .expect("invariant: refreshed lines have a key")
    }
}
