//! Role: Debug-build layout warnings for texts whose box or content changed.
//!
//! The layout warnings of debug builds: after a layout pass the texts
//! whose boxes (or content) changed are checked again, from the lines'
//! advances and metrics only (no glyph outlines are read), and each defect
//! found is queued as `HostEvent::LayoutWarning` (the node, the rule and
//! how far), under the rules and defaults of `LayoutCheck::default`. It runs
//! from `Model::settle`, between the layout pass and the sync after it.
//!
//! Invariants:
//! - A text is checked when its box key (its size and content box, and
//!   where it lies in its visual parent, with that parent's size and
//!   content box) differs from the one it was last checked with, or when
//!   its content was measured again (a write, an inherited style). A
//!   scroll or a transform changes no key.
//! - A defect is reported when its text is checked, so once per (node,
//!   rule) until the text's box or content changes; a text that is checked
//!   again and is still wrong is reported again.
//! - A node that is gone or not shown is forgotten.

mod changed_texts;
pub(crate) mod state;
