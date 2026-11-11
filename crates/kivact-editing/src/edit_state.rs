//! Role: `EditState`: a text, its revision and session, changed by recorded edits.

use crate::change::{Damage, TextChange};
use crate::session::TextSession;
use std::ops::Range;
use std::sync::atomic::{AtomicU64, Ordering};
use unicode_segmentation::UnicodeSegmentation as _;

/// The revision of a new text: unique for the process.
fn next_revision() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// A recorded change and the text it removed (to revert it).
#[derive(Clone, Debug)]
struct Logged {
    change: TextChange,
    removed: String,
}

/// How a state taken from kivact changed the editor's text
/// ([`Editor::incoming`](crate::Editor::incoming)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taken {
    /// The same text: only the session is taken, and the text keeps its
    /// revision (a state of the same text from another revision changes
    /// nothing that has to be reported or taken back).
    Same,
    /// By the changes the state recorded since it was read from this editor
    /// ([`Editor::snapshot`](crate::Editor::snapshot)).
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
    // xtask-allow(state-fields): option: absent means the editor has no scalar limit
    /// The scalar limit of the edits, if any.
    max_length: Option<usize>,
    // xtask-allow(state-fields): independent: the edits kivact has not taken yet (take_changes, start_log, revert clear it); the line-cache damage is cleared only by take_damage, and set_controlled leaves damage without a log, so all four combinations occur
    /// The changes recorded since the log started; `None` when they cover
    /// nothing, which is the same as "the log starts at `revision`".
    log: Option<Log>,
    // xtask-allow(state-fields): independent: line-cache damage is taken by the editor, separately from the edit log
    /// What changed since the editor's lines last took the damage.
    damage: Option<Damage>,
}

/// The changes recorded since `base`, never empty: a log of no change is
/// absent (`EditState::log`).
#[derive(Clone, Debug)]
struct Log {
    base: u64,
    entries: Vec<Logged>,
}

impl PartialEq for EditState {
    /// The same text and session; the texts are compared only when the
    /// revisions differ.
    fn eq(&self, other: &Self) -> bool {
        self.session == other.session
            && (self.revision == other.revision || self.text == other.text)
    }
}

/// The largest char boundary of `s` not after `x`.
pub(crate) use kivact_types::text_utf::floor_char_boundary as floor_boundary;

impl EditState {
    /// A state holding `value` with the caret at its end.
    pub fn new(value: &str) -> Self {
        let revision = next_revision();
        EditState {
            text: value.to_string(),
            revision,
            session: TextSession::caret_at(value.len()),
            max_length: None,
            log: None,
            damage: None,
        }
    }

    /// Set the scalar limit used by subsequent edits.
    pub(crate) fn set_max_length(&mut self, max_length: Option<usize>) {
        self.max_length = max_length;
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
    pub(crate) fn revision(&self) -> u64 {
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
        &self.text[self.session.selection()]
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
        kivact_types::text_utf::utf16_offset(&self.text, byte)
    }

    /// Byte offset of a UTF-16 offset (clamped to the value).
    pub fn byte_of_utf16(&self, units: usize) -> usize {
        kivact_types::text_utf::byte_offset_from_utf16(&self.text, units)
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
        self.session = TextSession {
            anchor: floor_boundary(&self.text, session.anchor),
            caret: floor_boundary(&self.text, session.caret),
            marked: session.marked.map(|m| self.clamp(m)),
        };
    }

    /// Replace the text with `value`, a value set by the core that the
    /// caller found different, keeping the session clamped into it. Held
    /// back while composing (the IME owns the text). Returns whether the
    /// value changed. Not recorded: it is no edit.
    pub(crate) fn set_controlled(&mut self, value: &str) -> bool {
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
    pub(crate) fn adopt(&mut self, value: &str) -> bool {
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
            self.session.caret = to;
        } else {
            self.session.anchor = to;
            self.session.caret = to;
        }
    }

    /// Replace the clamped range `r` with `text` and record it.
    fn splice(&mut self, r: Range<usize>, text: &str) {
        let removed = self.text[r.clone()].to_string();
        self.text.replace_range(r.clone(), text);
        let before = self.revision;
        self.revision = next_revision();
        self.damaged(before, r.clone(), text.len());
        let entries = vec![Logged {
            change: TextChange {
                range: r,
                text: text.to_string(),
            },
            removed,
        }];
        match &mut self.log {
            Some(log) => log.entries.extend(entries),
            None => {
                self.log = Some(Log {
                    base: before,
                    entries,
                });
            }
        }
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
    pub(crate) fn truncate(&mut self, end: usize) {
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
            .unwrap_or_else(|| self.session.selection());
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
                .unwrap_or_else(|| self.session.selection()),
        );
        let start = r.start;
        let limit = self.max_length;
        self.set_max_length(None);
        self.replace(r, text);
        self.set_max_length(limit);
        self.session.marked = (!text.is_empty()).then(|| start..start + text.len());
        let rel = selected.unwrap_or(text.len()..text.len());
        let rel = floor_boundary(text, rel.start)..floor_boundary(text, rel.end);
        self.session.anchor = start + rel.start;
        self.session.caret = start + rel.end.max(rel.start);
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
    pub(crate) fn start_log(&mut self) {
        self.log = None;
    }

    /// Whether this state recorded every change since the text at
    /// `revision` (it was read from a text at that revision).
    pub(crate) fn recorded_since(&self, revision: u64) -> bool {
        self.log.as_ref().map_or(self.revision, |log| log.base) == revision
    }

    /// Whether the log recorded a change since it started.
    pub(crate) fn has_changes(&self) -> bool {
        self.log.is_some()
    }

    /// The recorded changes, starting a new log.
    pub(crate) fn take_changes(&mut self) -> Vec<TextChange> {
        self.log
            .take()
            .map(|log| log.entries)
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.change)
            .collect()
    }

    /// Undo the recorded changes, back to the text at `revision` (where
    /// the log started), with `session`.
    pub(crate) fn revert(&mut self, revision: u64, session: TextSession) {
        let entries = self.log.take().map(|log| log.entries).unwrap_or_default();
        for logged in entries.into_iter().rev() {
            let start = logged.change.range.start;
            let range = start..start + logged.change.text.len();
            self.text.replace_range(range.clone(), &logged.removed);
            self.damaged(self.revision, range, logged.removed.len());
        }
        self.revision = revision;
        self.session = session;
    }

    /// Take `state`'s session, and its text and revision when it changed
    /// this text, recording how (`how`).
    pub(crate) fn take(&mut self, state: EditState, how: Taken) {
        let before = self.revision;
        match how {
            Taken::Same => {
                self.session = state.session;
                return;
            }
            Taken::Recorded => {
                let incoming = state.log;
                if let Some(log) = &incoming {
                    for logged in &log.entries {
                        let change = &logged.change;
                        self.damaged(before, change.range.clone(), change.text.len());
                    }
                }
                self.text = state.text;
                if let Some(log) = incoming {
                    match &mut self.log {
                        Some(mine) => mine.entries.extend(log.entries),
                        None => {
                            self.log = Some(log);
                        }
                    }
                }
            }
            Taken::Whole => {
                let old = std::mem::replace(&mut self.text, state.text);
                self.damaged(before, 0..old.len(), self.text.len());
                let entries = vec![Logged {
                    change: TextChange {
                        range: 0..old.len(),
                        text: self.text.clone(),
                    },
                    removed: old,
                }];
                match &mut self.log {
                    Some(log) => log.entries.extend(entries),
                    None => {
                        self.log = Some(Log {
                            base: before,
                            entries,
                        });
                    }
                }
            }
        }
        self.revision = state.revision;
        self.session = state.session;
    }

    /// `range` of the text at revision `before` (the current one when
    /// damage is already pending) was replaced by `len` bytes: join it to
    /// the damage.
    fn damaged(&mut self, before: u64, range: Range<usize>, len: usize) {
        let damage = match self.damage {
            Some(damage) => damage.then(range, len),
            None => Damage {
                base: before,
                start: range.start,
                old_end: range.end,
                new_end: range.start + len,
            },
        };
        self.damage = Some(damage);
    }

    /// What changed since the editor's lines last took it (`None`: nothing
    /// recorded since this state was made).
    pub(crate) fn take_damage(&mut self) -> Option<Damage> {
        self.damage.take()
    }
}
