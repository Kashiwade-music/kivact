//! Role: Store sparse layout interpolation runs.

use kivact_types::NodeId;
use kivact_types::motion::{SpringRun, Transition};

#[derive(Default)]
pub(crate) struct LayoutMotion(pub(super) std::collections::HashMap<NodeId, LayoutAnimation>);

pub(super) struct LayoutAnimation {
    pub(super) from: Vec<f64>,
    pub(super) to: Vec<f64>,
    pub(super) current: Vec<f64>,
    pub(super) timing: Transition,
    pub(super) start: u64,
    pub(super) spring: SpringRun,
}
