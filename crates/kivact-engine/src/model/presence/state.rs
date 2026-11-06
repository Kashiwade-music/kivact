//! Role: Track scheduling indexes for node entrance and exit transitions.

use std::collections::{BTreeMap, BTreeSet};

use kivact_types::NodeId;

/// The nodes awaiting attachment or completing entrance motion, and
/// the roots of the leaving subtrees.
#[derive(Default)]
pub(crate) struct Presences {
    pub(crate) sizes: BTreeMap<NodeId, SizeMotion>,
    pub(super) arriving: BTreeSet<NodeId>,
    pub(super) leaving: BTreeSet<NodeId>,
}

/// The scalar motion of one presence axis.
pub(crate) struct SizeMotion {
    pub(crate) axis: kivact_types::Orientation,
    pub(super) transition: kivact_types::motion::Transition,
    pub(super) start: u64,
    pub(super) from: f64,
    pub(super) to: f64,
    pub(super) current: Vec<f64>,
    pub(super) spring: kivact_types::motion::SpringRun,
}
