//! Role: The presence group's state: nodes awaiting their entrance and leaving subtrees.

use std::collections::BTreeSet;

use kivact_wire::NodeId;

/// The nodes between their creation and their first attached commit, and
/// the roots of the leaving subtrees.
#[derive(Default)]
pub(crate) struct Presences {
    pub(super) arriving: BTreeSet<NodeId>,
    pub(super) leaving: BTreeSet<NodeId>,
}
