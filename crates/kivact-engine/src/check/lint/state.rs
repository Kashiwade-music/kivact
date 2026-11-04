//! Role: The lint group's state: the texts checked and their box keys.

use kivact_types::{IdMap, LayoutCheck, NodeId, Point, Rect, Size};

/// The layout facts the rules depend on, all in the layout space (no
/// scroll offset or transform in them).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BoxKey {
    pub(super) size: Size,
    pub(super) content: Rect,
    /// Its visual parent, its size and content box, and where the text is
    /// in it.
    pub(super) parent: Option<(NodeId, Size, Rect, Point)>,
}

/// The texts checked so far (`Model::lint`).
#[derive(Debug, Default)]
pub(crate) struct Lint {
    pub(super) keys: IdMap<BoxKey>,
    pub(super) check: LayoutCheck,
}
