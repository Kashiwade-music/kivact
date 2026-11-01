//! Role: Typed editing state and text layout contracts.
//!
//! Owned native editing state, composition and shaped layout snapshots.
//! An [`EditState`] is a document (the text and its revision) with an
//! editing session over it ([`TextSession`]: the selection, the caret's
//! side and the IME composition). Every change of the text is a range
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
//! - `sel.start <= sel.end`; `reversed` says the caret is at `sel.start`.
//! - Every change of the text takes a new revision from one counter, so
//!   equal revisions mean equal text; comparing states compares their
//!   texts only when the revisions differ.
//! - The text changes only through recorded range replacements, kept with
//!   the text they removed (so the host can revert them), in order.
//! - Moving the caret or the selection never changes the text: such keys
//!   report `changed: false`, so the host sends no `TextChanged`.
//! - A value set by the core waits while a composition is in progress; the
//!   host gives it again when the composition ends, as a recorded change
//!   (`Editor::adopt`).
//! - The caret starts at the end of the value.
//! - Every change of the text, recorded or not, joins the state's damage:
//!   the one byte range that changed since the revision it starts from,
//!   which the editor's lines take to shape again only what changed
//!   (`text_lines`).
//! - An [`EditContext`] answers for the text the request found: once the
//!   behavior changed the state's text (a new revision), its layout
//!   queries answer `None`.
//! - Two behaviors, or two line colors, are equal when they are the same
//!   function object. A line's colors are clamped to the line's char
//!   boundaries, sorted and cut where they overlap.

#![forbid(unsafe_code)]

use std::any::Any;
use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use ui_types::{Key, Modifiers, Rect, Rgba};
use unicode_segmentation::UnicodeSegmentation as _;

/// The revision of a new text: unique for the process.
fn next_revision() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// An editing session over a text: the selection, which end holds the
/// caret, and the IME composition.
///
/// Offsets are UTF-8 byte offsets into the session's text. A session holds
/// no text, so reading it never copies the document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextSession {
    sel: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
}

impl TextSession {
    /// A caret at byte offset `at`, without a selection or composition.
    pub fn caret_at(at: usize) -> Self {
        TextSession {
            sel: at..at,
            reversed: false,
            marked: None,
        }
    }

    /// A selection from `anchor` to the caret at `caret` (byte offsets).
    pub fn select(anchor: usize, caret: usize) -> Self {
        TextSession {
            sel: anchor.min(caret)..anchor.max(caret),
            reversed: caret < anchor,
            marked: None,
        }
    }

    /// The selected byte range (empty: the caret).
    pub fn selection(&self) -> Range<usize> {
        self.sel.clone()
    }

    /// The caret's byte offset.
    pub fn caret(&self) -> usize {
        if self.reversed {
            self.sel.start
        } else {
            self.sel.end
        }
    }

    /// The selection's fixed end (the caret's other side).
    pub fn anchor(&self) -> usize {
        if self.reversed {
            self.sel.end
        } else {
            self.sel.start
        }
    }

    /// The composed (marked) byte range of an IME composition.
    pub fn marked(&self) -> Option<Range<usize>> {
        self.marked.clone()
    }

    /// An IME composition is in progress.
    pub fn is_composing(&self) -> bool {
        self.marked.is_some()
    }
}

/// One change of a text: the byte range `range` of the text before it was
/// replaced with `text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextChange {
    /// The replaced UTF-8 byte range of the text before the change.
    pub range: Range<usize>,
    /// What replaced it.
    pub text: String,
}

impl TextChange {
    /// Make the same change to `text`, a copy of the text it was made to.
    /// Returns `false`, changing nothing, when the range does not lie in
    /// `text` on char boundaries: `text` is not the text the change was
    /// made to (read the editor's to sync it again).
    pub fn apply(&self, text: &mut String) -> bool {
        if text.get(self.range.clone()).is_none() {
            return false;
        }
        text.replace_range(self.range.clone(), &self.text);
        true
    }
}

/// A recorded change and the text it removed (to revert it).
#[derive(Clone, Debug)]
struct Logged {
    change: TextChange,
    removed: String,
}

/// What changed in a text since the revision `base`: the bytes
/// `start..old_end` of the text at `base` became `start..new_end` of the
/// text now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Damage {
    /// Revision whose original byte range is replaced.
    pub base: u64,
    /// First changed byte in both versions.
    pub start: usize,
    /// End of the replaced range in the original revision.
    pub old_end: usize,
    /// End of the replacement range in the current revision.
    pub new_end: usize,
}

impl Damage {
    /// This damage, then `range` (of the text it leaves) replaced by
    /// `len` bytes.
    fn then(self, range: Range<usize>, len: usize) -> Self {
        let grown = range.end.saturating_sub(self.new_end);
        let end = self.new_end.max(range.end);
        Damage {
            base: self.base,
            start: self.start.min(range.start),
            old_end: self.old_end + grown,
            new_end: end + len - range.len(),
        }
    }
}

/// How a state taken from kivact changed the editor's text
/// ([`Editor::take`]).
pub enum Taken {
    /// The same text: only the session is taken, and the text keeps its
    /// revision (a state of the same text from another revision changes
    /// nothing that has to be reported or taken back).
    Same,
    /// By the changes the state recorded since it was read. Its journal
    /// must cover the receiving editor's current revision, as checked by
    /// [`Editor::recorded_since`] before the state is passed to [`Editor::take`].
    Recorded,
    /// Replaced as a whole.
    Whole,
}

/// An owned snapshot of a native editor: its document (the text and its
/// revision) and its [`TextSession`].
///
/// Byte ranges always fall on UTF-8 character boundaries. Use the UTF-16
/// conversion methods at platform IME boundaries. A new state starts with
/// its caret at the end; editing mutations preserve these invariants.
/// Changes of the text are recorded as range replacements: a state read
/// from an editor, changed and given back changes only those ranges.
#[derive(Clone, Debug, Default)]
pub struct EditState {
    text: String,
    revision: u64,
    session: TextSession,
    max_length: Option<usize>,
    /// The revision the log starts from.
    base: u64,
    log: Vec<Logged>,
    /// What changed since the editor's lines last took the damage.
    damage: Cell<Option<Damage>>,
}

/// A native host's editor, owning the behavior-facing document and its journal.
///
/// Behaviors receive only the state. The host controls recording, controlled
/// values and pending damage through this owner. Cloning copies the document,
/// journal and damage independently; no mutable editor state is shared.
#[derive(Clone, Debug)]
pub struct Editor {
    state: EditState,
}

impl PartialEq for EditState {
    /// The same text and session; the texts are compared only when the
    /// revisions differ.
    fn eq(&self, other: &Self) -> bool {
        self.session == other.session
            && (self.revision == other.revision || self.text == other.text)
    }
}

/// The editing behavior's result, interpreted by the native host.
#[derive(Clone, Debug, PartialEq)]
pub enum EditOutcome {
    /// Leave this request for normal event routing.
    NotEditing,
    /// Consumed; `changed` says the value changed (send `TextChanged`).
    Done {
        /// The value changed.
        changed: bool,
    },
    /// Copy or cut of a non-empty selection: the host writes `text` to the
    /// clipboard (a cut already removed it: `changed`).
    Copy {
        /// The copied text.
        text: String,
        /// The value changed (a cut).
        changed: bool,
    },
    /// Ask the host to read the clipboard and route a Paste request.
    Paste,
}

/// The largest char boundary of `s` not after `x`.
fn floor_boundary(s: &str, x: usize) -> usize {
    let mut x = x.min(s.len());
    while !s.is_char_boundary(x) {
        x -= 1;
    }
    x
}

impl EditState {
    /// A state holding `value` with the caret at its end.
    pub fn new(value: &str) -> Self {
        let revision = next_revision();
        EditState {
            text: value.to_string(),
            revision,
            session: TextSession::caret_at(value.len()),
            max_length: None,
            base: revision,
            log: Vec::new(),
            damage: Cell::new(None),
        }
    }

    /// The value.
    pub fn value(&self) -> &str {
        &self.text
    }

    /// The value, without copying it.
    pub fn into_value(self) -> String {
        self.text
    }

    /// The editing session: selection, caret and composition.
    pub fn session(&self) -> &TextSession {
        &self.session
    }

    /// The text's revision: equal revisions mean equal text.
    fn revision(&self) -> u64 {
        self.revision
    }

    /// The selected byte range (empty: the caret).
    pub fn selection(&self) -> Range<usize> {
        self.session.selection()
    }

    /// The caret's byte offset.
    pub fn caret(&self) -> usize {
        self.session.caret()
    }

    /// The selection's fixed end (the caret's other side).
    pub fn anchor(&self) -> usize {
        self.session.anchor()
    }

    /// The composed (marked) byte range of an IME composition.
    pub fn marked(&self) -> Option<Range<usize>> {
        self.session.marked()
    }

    /// An IME composition is in progress.
    pub fn is_composing(&self) -> bool {
        self.session.is_composing()
    }

    /// The selected text.
    pub fn selected_text(&self) -> &str {
        &self.text[self.session.sel.clone()]
    }

    /// Char offset of a byte offset. It counts the text before it.
    pub fn char_of_byte(&self, byte: usize) -> u32 {
        self.text[..floor_boundary(&self.text, byte)]
            .chars()
            .count() as u32
    }

    /// Byte offset of a char offset (clamped to the value).
    pub fn byte_of_char(&self, chars: u32) -> usize {
        self.text
            .char_indices()
            .nth(chars as usize)
            .map_or(self.text.len(), |(i, _)| i)
    }

    /// UTF-16 offset of a byte offset (platform IMEs count UTF-16 units).
    pub fn utf16_of_byte(&self, byte: usize) -> usize {
        self.text[..floor_boundary(&self.text, byte)]
            .encode_utf16()
            .count()
    }

    /// Byte offset of a UTF-16 offset (clamped to the value).
    pub fn byte_of_utf16(&self, units: usize) -> usize {
        let mut seen = 0;
        for (i, c) in self.text.char_indices() {
            if seen >= units {
                return i;
            }
            seen += c.len_utf16();
        }
        self.text.len()
    }

    /// The grapheme boundary before `at` (0 at the start).
    pub fn prev_grapheme(&self, at: usize) -> usize {
        // Boundaries before `at` depend only on the text up to it.
        let at = floor_boundary(&self.text, at);
        self.text[..at]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(i, _)| i)
    }

    /// The grapheme boundary after `at` (the length at the end).
    pub fn next_grapheme(&self, at: usize) -> usize {
        // From a boundary, the next one depends only on the text after it.
        let at = floor_boundary(&self.text, at);
        self.text[at..]
            .grapheme_indices(true)
            .nth(1)
            .map_or(self.text.len(), |(i, _)| at + i)
    }

    fn clamp(&self, r: Range<usize>) -> Range<usize> {
        let start = floor_boundary(&self.text, r.start);
        start..floor_boundary(&self.text, r.end).max(start)
    }

    /// Replace the session with `session`, clamped into the text (on char
    /// boundaries).
    pub fn set_session(&mut self, session: TextSession) {
        let sel = self.clamp(session.sel);
        let marked = session.marked.map(|m| self.clamp(m));
        self.session = TextSession {
            reversed: session.reversed && !sel.is_empty(),
            sel,
            marked,
        };
    }

    /// Replace the text with `value`, a value set by the core that the
    /// caller found different, keeping the session clamped into it. Held
    /// back while composing (the IME owns the text). Returns whether the
    /// value changed. Not recorded: it is no edit.
    fn set_controlled(&mut self, value: &str) -> bool {
        if self.session.marked.is_some() {
            return false;
        }
        let session = self.session.clone();
        let before = (self.revision, self.text.len());
        value.clone_into(&mut self.text);
        self.revision = next_revision();
        self.damaged(before.0, 0..before.1, self.text.len());
        self.start_log();
        self.set_session(session);
        true
    }

    /// Take `value`, a value set by the core that waited for a
    /// composition to end, as one recorded change of the part that
    /// differs, keeping the session clamped into it. Returns whether the
    /// text changed.
    fn adopt(&mut self, value: &str) -> bool {
        if self.text == value {
            return false;
        }
        let head = self
            .text
            .char_indices()
            .zip(value.chars())
            .take_while(|((_, a), b)| a == b)
            .last()
            .map_or(0, |((at, c), _)| at + c.len_utf8());
        let rest = self.text.len().min(value.len()) - head;
        let tail = self.text[head..]
            .chars()
            .rev()
            .zip(value[head..].chars().rev())
            .take_while(|(a, b)| a == b)
            .map(|(c, _)| c.len_utf8())
            .sum::<usize>()
            .min(rest);
        let session = self.session.clone();
        let (end, with) = (self.text.len() - tail, &value[head..value.len() - tail]);
        self.splice(head..end, with);
        self.set_session(session);
        true
    }

    /// Put the caret at `to`, extending the selection from its anchor with
    /// `extend` (Shift, or a drag).
    pub fn move_to(&mut self, to: usize, extend: bool) {
        let to = floor_boundary(&self.text, to);
        if extend {
            let anchor = self.anchor();
            self.session.sel = anchor.min(to)..anchor.max(to);
            self.session.reversed = to < anchor;
        } else {
            self.session.sel = to..to;
            self.session.reversed = false;
        }
    }

    /// Replace the clamped range `r` with `text` and record it.
    fn splice(&mut self, r: Range<usize>, text: &str) {
        let removed = self.text[r.clone()].to_string();
        self.text.replace_range(r.clone(), text);
        let before = self.revision;
        self.revision = next_revision();
        self.damaged(before, r.clone(), text.len());
        self.log.push(Logged {
            change: TextChange {
                range: r,
                text: text.to_string(),
            },
            removed,
        });
    }

    /// Replace `range` with `text`; the caret follows it and any
    /// composition ends.
    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        let r = self.clamp(range);
        let end = self
            .max_length
            .map(|max| {
                let remaining = self.text.chars().count() - self.text[r.clone()].chars().count();
                text.char_indices()
                    .nth(max.saturating_sub(remaining))
                    .map_or(text.len(), |(byte, _)| byte)
            })
            .unwrap_or(text.len());
        let text = &text[..end];
        if !(r.is_empty() && text.is_empty()) {
            self.splice(r.clone(), text);
        }
        self.session = TextSession::caret_at(r.start + text.len());
    }

    /// Remove the text after byte offset `end` (a length limit), keeping
    /// the session clamped into what is left.
    fn truncate(&mut self, end: usize) {
        let end = floor_boundary(&self.text, end);
        if end == self.text.len() {
            return;
        }
        let session = self.session.clone();
        self.splice(end..self.text.len(), "");
        self.set_session(session);
    }

    /// Text committed by typing or an IME: replaces `range`, else the
    /// composition, else the selection.
    pub fn commit(&mut self, range: Option<Range<usize>>, text: &str) {
        let r = range
            .or_else(|| self.session.marked.clone())
            .unwrap_or_else(|| self.session.sel.clone());
        self.replace(r, text);
    }

    /// An IME composition: `text` replaces `range` (else the composition,
    /// else the selection) and is marked; `selected` is the selection
    /// inside `text` in bytes (the caret after it without one).
    pub fn compose(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
    ) {
        let r = self.clamp(
            range
                .or_else(|| self.session.marked.clone())
                .unwrap_or_else(|| self.session.sel.clone()),
        );
        let start = r.start;
        let limit = self.max_length.take();
        self.replace(r, text);
        self.max_length = limit;
        self.session.marked = (!text.is_empty()).then(|| start..start + text.len());
        let rel = selected.unwrap_or(text.len()..text.len());
        let rel = floor_boundary(text, rel.start)..floor_boundary(text, rel.end);
        self.session.sel = start + rel.start..start + rel.end.max(rel.start);
    }

    /// End the composition, keeping as much of its text as the length
    /// limit allows, with the caret after it. Returns whether one ended.
    pub fn unmark(&mut self) -> bool {
        let Some(marked) = self.session.marked.take() else {
            return false;
        };
        let mut end = marked.end;
        if let Some(max) = self.max_length {
            let composed = self.text[marked.clone()].chars().count();
            let others = self.text.chars().count() - composed;
            let keep = max.saturating_sub(others);
            if keep < composed {
                end = self.text[marked.clone()]
                    .char_indices()
                    .nth(keep)
                    .map_or(marked.end, |(byte, _)| marked.start + byte);
                self.splice(end..marked.end, "");
            }
        }
        self.session = TextSession::caret_at(end);
        true
    }

    /// Start recording changes from the current text.
    fn start_log(&mut self) {
        self.log.clear();
        self.base = self.revision;
    }

    /// Whether this state recorded every change since the text at
    /// `revision` (it was read from a text at that revision).
    fn recorded_since(&self, revision: u64) -> bool {
        self.base == revision
    }

    /// The recorded changes, starting a new log.
    fn take_changes(&mut self) -> Vec<TextChange> {
        let log = std::mem::take(&mut self.log);
        self.base = self.revision;
        log.into_iter().map(|l| l.change).collect()
    }

    /// Undo the recorded changes, back to the text at `revision` (where
    /// the log started), with `session`.
    fn revert(&mut self, revision: u64, session: TextSession) {
        for logged in std::mem::take(&mut self.log).into_iter().rev() {
            let start = logged.change.range.start;
            let range = start..start + logged.change.text.len();
            self.text.replace_range(range.clone(), &logged.removed);
            self.damaged(self.revision, range, logged.removed.len());
        }
        self.revision = revision;
        self.base = revision;
        self.session = session;
    }

    /// Take `state`'s session, and its text and revision when it changed
    /// this text, recording how (`how`).
    fn take(&mut self, state: EditState, how: Taken) {
        let before = self.revision;
        match how {
            Taken::Same => {
                self.session = state.session;
                return;
            }
            Taken::Recorded => {
                for logged in &state.log {
                    let change = &logged.change;
                    self.damaged(before, change.range.clone(), change.text.len());
                }
                self.text = state.text;
                self.log.extend(state.log);
            }
            Taken::Whole => {
                let old = std::mem::replace(&mut self.text, state.text);
                self.damaged(before, 0..old.len(), self.text.len());
                self.log.push(Logged {
                    change: TextChange {
                        range: 0..old.len(),
                        text: self.text.clone(),
                    },
                    removed: old,
                });
            }
        }
        self.revision = state.revision;
        self.session = state.session;
    }

    /// `range` of the text at revision `before` (the current one when
    /// damage is already pending) was replaced by `len` bytes: join it to
    /// the damage.
    fn damaged(&self, before: u64, range: Range<usize>, len: usize) {
        let damage = match self.damage.get() {
            Some(damage) => damage.then(range, len),
            None => Damage {
                base: before,
                start: range.start,
                old_end: range.end,
                new_end: range.start + len,
            },
        };
        self.damage.set(Some(damage));
    }

    /// What changed since the editor's lines last took it (`None`: nothing
    /// recorded since this state was made).
    fn take_damage(&self) -> Option<Damage> {
        self.damage.take()
    }
}

/// Shaped editing geometry of some of an editor's lines (those asked for),
/// in logical window coordinates after scroll and transforms. Rectangles
/// are the outer bounds of transformed boxes.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    /// The active selection end, at the same position used by the native IME.
    pub caret: Rect,
    /// How many lines the editor shows (0 while it shows its placeholder);
    /// native editors break at line feeds.
    pub line_count: usize,
    /// The requested lines that exist, in reading order.
    pub lines: Vec<EditLine>,
    /// The caret stops of those lines, with Unicode scalar offsets in the
    /// whole value.
    pub carets: Vec<EditCaret>,
}

/// The original text range and geometry of one native editing line.
#[derive(Clone, Debug, PartialEq)]
pub struct EditLine {
    /// UTF-8 byte range in the original value, including its trailing break.
    pub range: Range<usize>,
    /// The line's transformed outer bounds in logical window coordinates.
    pub rect: Rect,
    /// Window y coordinate of the baseline at the line origin.
    pub baseline: f32,
}

/// One shaped caret stop in the editor's original value.
#[derive(Clone, Debug, PartialEq)]
pub struct EditCaret {
    /// Unicode scalar offset, including the final boundary.
    pub offset: u32,
    /// The caret's transformed outer bounds in logical window coordinates.
    pub rect: Rect,
}

/// What an editing behavior is given beside the state and the request:
/// the editor's native constraints and its laid-out lines.
///
/// The layout queries take the state the behavior was given and answer
/// for its text as the request found it (`None` once the behavior changed
/// the text, or when the editor is not laid out or shows its
/// placeholder). Lines are hard lines (a single-line editor has one);
/// `x` is in logical pixels from the start of the editor's text box,
/// before the editor's horizontal scroll.
pub struct EditContext<'a> {
    /// Whether committed text may contain line breaks.
    pub multiline: bool,
    /// Whether user input must preserve the value.
    pub read_only: bool,
    /// Whether clipboard copy and cut must hide the value.
    pub password: bool,
    layout: Option<EditLayout<'a>>,
}

/// The host's laid-out lines, as an [`EditContext`] reads them.
/// Offsets and ranges are UTF-8 byte positions in the supplied document;
/// content ranges exclude line breaks. A break belongs to the line it ends,
/// and the byte immediately after the break belongs to the next line.
pub trait Lines {
    /// They show the value, not the placeholder.
    fn shows_value(&self) -> bool;
    /// How many there are.
    fn len(&self) -> usize;
    /// Whether this provider has no lines.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The line holding byte offset `at`, including its terminating break.
    fn line_of(&self, at: usize) -> usize;
    /// Line `i`'s text, without its break.
    fn content(&self, i: usize) -> Option<Range<usize>>;
    /// The caret at byte offset `at` of `text`, in a box `width` wide.
    fn caret(&self, text: &str, at: usize, width: f32) -> Rect;
    /// The grapheme boundary of line `i` whose caret is nearest `x`.
    fn nearest(&self, text: &str, i: usize, x: f32, width: f32) -> usize;
}

/// The lines an [`EditContext`] reads, the revision of the text they are
/// of and the width of the box they are aligned in.
#[derive(Clone, Copy)]
pub struct EditLayout<'a> {
    /// Host line geometry provider for this snapshot.
    pub lines: &'a (dyn Lines + 'a),
    /// Document revision for which the geometry was computed.
    pub revision: u64,
    /// Logical alignment width before horizontal text scrolling.
    pub width: f32,
}

impl<'a> EditContext<'a> {
    /// Build the constraints and optional geometry for one editing request.
    /// Geometry queries remain valid only for the supplied layout's revision.
    pub fn new(
        multiline: bool,
        read_only: bool,
        password: bool,
        layout: Option<EditLayout<'a>>,
    ) -> Self {
        Self {
            multiline,
            read_only,
            password,
            layout,
        }
    }

    /// The lines, when they are of `state`'s text.
    fn layout(&self, state: &EditState) -> Option<EditLayout<'a>> {
        self.layout
            .filter(|l| l.revision == state.revision() && l.lines.shows_value())
    }

    /// The number of lines of `state`'s text.
    pub fn line_count(&self, state: &EditState) -> Option<usize> {
        Some(self.layout(state)?.lines.len())
    }

    /// The line holding byte offset `at` of `state`'s text (a line break
    /// belongs to the line it ends).
    pub fn line_of(&self, state: &EditState, at: usize) -> Option<usize> {
        Some(self.layout(state)?.lines.line_of(at))
    }

    /// The byte range of `line`'s text, without its line break.
    pub fn line_range(&self, state: &EditState, line: usize) -> Option<Range<usize>> {
        self.layout(state)?.lines.content(line)
    }

    /// Where the caret at byte offset `at` of `state`'s text is drawn: its
    /// `x`.
    pub fn x_of(&self, state: &EditState, at: usize) -> Option<f32> {
        let l = self.layout(state)?;
        Some(l.lines.caret(state.value(), at, l.width).x)
    }

    /// The grapheme boundary of `line` whose caret is nearest `x` (the
    /// line's end past its last character), as a byte offset of
    /// `state`'s text.
    pub fn offset_at(&self, state: &EditState, line: usize, x: f32) -> Option<usize> {
        let l = self.layout(state)?;
        (line < l.lines.len()).then(|| l.lines.nearest(state.value(), line, x, l.width))
    }
}

/// A routed key or clipboard text for the attached editing behavior.
pub enum EditRequest<'a> {
    /// A key after native composition and focus routing.
    Key {
        /// The logical editing key.
        key: Key,
        /// Modifiers held for this request.
        modifiers: Modifiers,
    },
    /// Clipboard text to normalize and insert.
    Paste(&'a str),
}

/// What an editing behavior runs.
type Behave = dyn Fn(&mut EditState, EditRequest<'_>, &EditContext<'_>) -> EditOutcome;

/// An editing function run synchronously while the engine owns its model.
///
/// The function changes the supplied EditState and may use state it
/// captured; it must not touch signals or the engine tree. Native IME
/// transport, shaping, pointer selection and clipboard services remain
/// with the host. Two behaviors are equal when they are the same function
/// object.
#[derive(Clone)]
pub struct EditBehavior(Rc<Behave>);
impl EditBehavior {
    /// Use this function for routed editing keys and clipboard text.
    pub fn new(
        f: impl Fn(&mut EditState, EditRequest<'_>, &EditContext<'_>) -> EditOutcome + 'static,
    ) -> Self {
        Self(Rc::new(f))
    }
    /// Invoke the behavior synchronously on the supplied document and request.
    /// The caller owns host services and handles the returned editing outcome.
    pub fn invoke(
        &self,
        state: &mut EditState,
        request: EditRequest<'_>,
        context: &EditContext<'_>,
    ) -> EditOutcome {
        (self.0)(state, request, context)
    }
}
impl PartialEq for EditBehavior {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl std::fmt::Debug for EditBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EditBehavior(..)")
    }
}

/// A value a [`LineState`] holds: compared by value.
trait StateValue {
    fn as_any(&self) -> &dyn Any;
    fn same(&self, other: &dyn StateValue) -> bool;
}

impl<S: PartialEq + 'static> StateValue for S {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn same(&self, other: &dyn StateValue) -> bool {
        other.as_any().downcast_ref::<S>() == Some(self)
    }
}

/// What coloring one line leaves for the next (a block comment still
/// open, say): opaque to the engine, which compares it to know whether
/// the lines after a changed line must be colored again. The first line
/// starts from the default (empty) state.
#[derive(Clone, Default)]
pub struct LineState(Option<Rc<dyn StateValue>>);

impl LineState {
    /// A state holding `state`.
    pub fn new<S: PartialEq + 'static>(state: S) -> Self {
        Self(Some(Rc::new(state)))
    }

    /// The value this state holds, if it holds an `S` (`None` for the
    /// first line's state).
    pub fn get<S: 'static>(&self) -> Option<&S> {
        self.0.as_ref()?.as_any().downcast_ref()
    }
}

impl PartialEq for LineState {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b) || a.same(&**b),
            _ => false,
        }
    }
}

impl std::fmt::Debug for LineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_some() {
            "LineState(..)"
        } else {
            "LineState(start)"
        })
    }
}

/// What line colors run: the colored byte ranges of one line, which
/// excludes its line break, from the state the line before left, and the
/// state it leaves.
type Colorize = dyn Fn(&str, &LineState) -> (Vec<(Range<usize>, Rgba)>, LineState);

/// Colors for the lines of a native editor's text, computed by kivact and
/// run synchronously while the engine draws the editor. Two are equal when
/// they are the same function object.
#[derive(Clone)]
pub struct LineColors(Rc<Colorize>);
impl LineColors {
    /// Color each line with `f`: `(range, color)` in the line and the
    /// state the line leaves, from the state the line before left.
    pub fn new(
        f: impl Fn(&str, &LineState) -> (Vec<(Range<usize>, Rgba)>, LineState) + 'static,
    ) -> Self {
        Self(Rc::new(f))
    }

    /// The sorted, non-overlapping colored byte ranges of `line` (clamped
    /// to its char boundaries) and the state it leaves, from `input`.
    pub fn line(&self, line: &str, input: &LineState) -> (Vec<(Range<usize>, Rgba)>, LineState) {
        let (mut ranges, output) = (self.0)(line, input);
        ranges.sort_by_key(|(r, _)| r.start);
        let mut out = Vec::with_capacity(ranges.len());
        let mut end = 0;
        for (r, color) in ranges {
            let a = floor_boundary(line, r.start).max(end);
            let b = floor_boundary(line, r.end);
            if a < b {
                out.push((a..b, color));
                end = b;
            }
        }
        (out, output)
    }
}
impl PartialEq for LineColors {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl std::fmt::Debug for LineColors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LineColors(..)")
    }
}

mod editor;
mod geometry;

pub use geometry::{TextGeometry, caret_at};
