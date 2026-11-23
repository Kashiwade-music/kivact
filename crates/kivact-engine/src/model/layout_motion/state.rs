//! Role: Store sparse layout interpolation runs and the boxes of departed shared layouts.

use std::collections::HashMap;

use kivact_types::NodeId;
use kivact_types::cell::ReportOnceSet;

use super::shared::SharedLayouts;
use crate::model::motion_run::MotionRun;

/// The layout motion group: the runs of the nodes whose boxes move, and the
/// window boxes of removed nodes that a shared layout may still take over.
#[derive(Default)]
pub(crate) struct LayoutMotion {
    /// The run of each node whose window box is moving (the window boxes
    /// `x, y, w, h` it moves from and to).
    pub(super) runs: HashMap<NodeId, MotionRun>,
    /// The boxes of removed nodes with a layout id, for the next layout pass.
    pub(super) shared: SharedLayouts,
    /// The layout ids already reported as duplicates.
    pub(super) reported_ids: ReportOnceSet<std::sync::Arc<str>>,
}
