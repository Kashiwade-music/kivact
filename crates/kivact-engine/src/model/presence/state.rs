//! Role: Track the size motion of node entrance and exit transitions.
//!
//! A node's presence itself is kept by the node tree (`NodeTree::presence_of`);
//! the nodes awaiting attachment and the leaving roots are read from it.

use std::collections::BTreeMap;

use kivact_types::NodeId;

/// The scalar size motion of entrances and exits, by node.
#[derive(Default)]
pub(crate) struct Presences {
    pub(crate) sizes: BTreeMap<NodeId, SizeMotion>,
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
