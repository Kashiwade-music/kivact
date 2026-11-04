//! Role: Application-owned bindings for this component view state.
use kivact::geometry::Point;
use kivact::state::{Binding, signal};
/// The view state of a [`timeline`](crate::timeline) or a
/// [`dope_sheet`](crate::dope_sheet).
#[derive(Clone, Copy)]
pub struct TimeState {
    /// The selected item IDs.
    pub selection: Binding<Vec<u64>>,
    /// The playhead, in frames.
    pub playhead: Binding<f64>,
    /// The horizontal scale, inside the editor's zoom range.
    pub zoom: Binding<f64>,
    /// The scroll offset in logical pixels.
    pub pan: Binding<Point>,
}

impl TimeState {
    /// Nothing selected, the playhead at 0, a scale of 1 and no scroll.
    /// Setup only.
    #[track_caller]
    pub fn new() -> Self {
        Self {
            selection: signal(Vec::new()).into(),
            playhead: signal(0.0).into(),
            zoom: signal(1.0).into(),
            pan: signal(Point::new(0.0, 0.0)).into(),
        }
    }
}

impl Default for TimeState {
    /// Same as [`TimeState::new`]. Setup only.
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}
