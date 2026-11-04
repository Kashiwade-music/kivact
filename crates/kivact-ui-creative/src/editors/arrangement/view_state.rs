//! Role: Application-owned bindings for this component view state.
use crate::editors::arrangement::model::TimelineTool;
use kivact::geometry::Point;
use kivact::state::{Binding, signal};
/// The view state of an [`arrangement`](crate::editors::arrangement).
#[derive(Clone, Copy)]
pub struct ArrangementState {
    /// The selected clip IDs.
    pub selection: Binding<Vec<u64>>,
    /// The playhead, in beats.
    pub playhead: Binding<f64>,
    /// The loop locators, start before end, in beats.
    pub loop_range: Binding<(f64, f64)>,
    /// Whether the loop is on.
    pub cycling: Binding<bool>,
    /// Logical pixels per beat, inside the arrangement's zoom range.
    pub zoom: Binding<f64>,
    /// The scroll offset in logical pixels.
    pub pan: Binding<Point>,
    /// The editing tool.
    pub tool: Binding<TimelineTool>,
}

impl ArrangementState {
    /// Nothing selected, the playhead at 0, a cycling loop over beats 0 to
    /// 16, 26 pixels per beat, no scroll and the select tool. Setup only.
    #[track_caller]
    pub fn new() -> Self {
        Self {
            selection: signal(Vec::new()).into(),
            playhead: signal(0.0).into(),
            loop_range: signal((0.0, 16.0)).into(),
            cycling: signal(true).into(),
            zoom: signal(26.0).into(),
            pan: signal(Point::new(0.0, 0.0)).into(),
            tool: signal(TimelineTool::Select).into(),
        }
    }
}

impl Default for ArrangementState {
    /// Same as [`ArrangementState::new`]. Setup only.
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}
