//! Role: Application-owned bindings for this component view state.
use crate::editors::arrangement::model::TimelineTool;
use kivact::interaction::ZoomPan;
use kivact::state::Signal;
use kivact::state::signal;
/// The view state of an [`arrangement`](crate::editors::arrangement).
#[derive(Clone, Copy)]
pub struct ArrangementState {
    /// The selected clip IDs.
    pub selection: Signal<Vec<u64>>,
    /// The playhead, in beats.
    pub playhead: Signal<f64>,
    /// The loop locators, start before end, in beats.
    pub loop_range: Signal<(f64, f64)>,
    /// Whether the loop is on.
    pub cycling: Signal<bool>,
    /// The shared view pose. Scale is horizontal pixels per logical time unit;
    /// offset is scrolling in logical pixels. Lane heights remain unscaled.
    pub transform: Signal<ZoomPan>,
    /// The editing tool.
    pub tool: Signal<TimelineTool>,
}

impl ArrangementState {
    /// Nothing selected, the playhead at 0, a cycling loop over beats 0 to
    /// 16, 26 pixels per beat, no scroll and the select tool. Setup only.
    #[track_caller]
    pub fn new() -> Self {
        Self {
            selection: signal(Vec::new()),
            playhead: signal(0.0),
            loop_range: signal((0.0, 16.0)),
            cycling: signal(true),
            transform: signal(ZoomPan {
                scale: 26.0,
                ..ZoomPan::default()
            }),
            tool: signal(TimelineTool::Select),
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
