//! Role: Application-owned bindings for this component view state.
use kivact::state::Signal;
use kivact::state::signal;
/// The view state of an [`outliner`](crate::outliner).
#[derive(Clone, Copy)]
pub struct OutlinerState {
    /// The IDs of the open items.
    pub expanded: Signal<Vec<u64>>,
    /// The selected item IDs.
    pub selection: Signal<Vec<u64>>,
    /// The item whose name is being edited, if any.
    pub renaming: Signal<Option<u64>>,
}

impl OutlinerState {
    /// The items `expanded` open, nothing selected and nothing renamed.
    /// Setup only.
    #[track_caller]
    pub fn new(expanded: impl IntoIterator<Item = u64>) -> Self {
        Self {
            expanded: signal(expanded.into_iter().collect()),
            selection: signal(Vec::new()),
            renaming: signal(None),
        }
    }
}
