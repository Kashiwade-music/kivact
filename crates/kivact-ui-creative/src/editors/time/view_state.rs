//! Role: Application-owned bindings for this component view state.
use kivact::interaction::ViewTransform;
use kivact::state::IntoBinding;
use kivact::state::{Binding, signal};
/// The view state of a [`timeline`](crate::timeline) or a
/// [`dope_sheet`](crate::dope_sheet).
#[derive(Clone, Copy)]
pub struct TimeState {
    /// The selected item IDs.
    pub selection: Binding<Vec<u64>>,
    /// The playhead, in frames.
    pub playhead: Binding<f64>,
    /// The shared view pose. Scale is horizontal pixels per logical time unit;
    /// offset is scrolling in logical pixels. Lane heights remain unscaled.
    pub transform: Binding<ViewTransform>,
}

impl TimeState {
    /// Nothing selected, the playhead at 0, a scale of 1 and no scroll.
    /// Setup only.
    #[track_caller]
    pub fn new() -> Self {
        Self {
            selection: signal(Vec::new()).into_binding(),
            playhead: signal(0.0).into_binding(),
            transform: signal(ViewTransform {
                scale: 1.0,
                ..ViewTransform::default()
            })
            .into_binding(),
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
