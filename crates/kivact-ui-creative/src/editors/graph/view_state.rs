//! Role: Application-owned bindings for this component view state.
use crate::editors::graph::model::GraphView;
use kivact::{Binding, signal};
/// The view state of a [`graph_editor`](crate::editors::graph).
#[derive(Clone, Copy)]
pub struct GraphState {
    /// The time and value ranges the graph shows.
    pub view: Binding<GraphView>,
    /// The selected key IDs.
    pub selection: Binding<Vec<u64>>,
    /// The playhead, in frames.
    pub playhead: Binding<f64>,
}

impl GraphState {
    /// The default [`GraphView`], nothing selected and the playhead at 0.
    /// Setup only.
    #[track_caller]
    #[allow(
        clippy::new_without_default,
        reason = "creation makes signals under the current setup owner; a `Default` impl would hide that"
    )]
    pub fn new() -> Self {
        Self {
            view: signal(GraphView::default()).into(),
            selection: signal(Vec::new()).into(),
            playhead: signal(0.0).into(),
        }
    }
}
