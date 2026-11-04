//! Role: Track scheduling indexes for node entrance and exit transitions.

use std::collections::BTreeSet;

use kivact_types::NodeId;

/// The nodes awaiting attachment or completing entrance motion, and
/// the roots of the leaving subtrees.
#[derive(Default)]
pub(crate) struct Presences {
    pub(super) arriving: BTreeSet<NodeId>,
    pub(super) leaving: BTreeSet<NodeId>,
}
