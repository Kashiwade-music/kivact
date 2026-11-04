//! Role: Application-owned bindings for this component view state.
use kivact::state::IntoBinding;
use kivact::state::{Binding, signal};
/// The view state of an [`outliner`](crate::outliner).
#[derive(Clone, Copy)]
pub struct OutlinerState {
    /// The IDs of the open items.
    pub expanded: Binding<Vec<u64>>,
    /// The selected item IDs.
    pub selection: Binding<Vec<u64>>,
    /// The item whose name is being edited, if any.
    pub renaming: Binding<Option<u64>>,
}

impl OutlinerState {
    /// The items `expanded` open, nothing selected and nothing renamed.
    /// Setup only.
    #[track_caller]
    pub fn new(expanded: impl IntoIterator<Item = u64>) -> Self {
        Self {
            expanded: signal(expanded.into_iter().collect()).into_binding(),
            selection: signal(Vec::new()).into_binding(),
            renaming: signal(None).into_binding(),
        }
    }
}
