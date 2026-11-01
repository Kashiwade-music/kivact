//! Role: Recorded range replacements of a text and the damage they leave.

use std::ops::Range;

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
    pub(crate) fn then(self, range: Range<usize>, len: usize) -> Self {
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
