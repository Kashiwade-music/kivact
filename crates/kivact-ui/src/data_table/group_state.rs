//! Role: Expose the group state shared by table recipes and rendering.
/// The state a group-header recipe draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataTableGroupState {
    /// Matching rows in this group on the current page, including collapsed rows.
    pub count: usize,
    /// Whether the group is collapsed.
    pub collapsed: bool,
}
