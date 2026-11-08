//! Role: Store sparse layout interpolation runs and the boxes of departed shared layouts.

use std::collections::HashMap;

use kivact_types::NodeId;
use kivact_types::cell::ReportOnceSet;
use kivact_types::motion::{SpringRun, Transition};

use super::shared::SharedLayouts;

/// The layout motion group: the runs of the nodes whose boxes move, and the
/// window boxes of removed nodes that a shared layout may still take over.
#[derive(Default)]
pub(crate) struct LayoutMotion {
    /// The run of each node whose window box is moving.
    pub(super) runs: HashMap<NodeId, LayoutAnimation>,
    /// The boxes of removed nodes with a layout id, for the next layout pass.
    pub(super) shared: SharedLayouts,
    /// The layout ids already reported as duplicates.
    pub(super) reported_ids: ReportOnceSet<std::sync::Arc<str>>,
}

/// One layout run: the window boxes (x, y, w, h) it moves from and to.
pub(super) struct LayoutAnimation {
    /// The window box the run starts from.
    pub(super) from: Vec<f64>,
    /// The window box the run goes to (the goal).
    pub(super) to: Vec<f64>,
    /// The window box drawn now.
    pub(super) current: Vec<f64>,
    /// The timing of the run (it plays once, forward).
    pub(super) timing: Transition,
    /// When the run started (host ns).
    pub(super) start: u64,
    /// The spring's velocity and step state.
    pub(super) spring: SpringRun,
}
