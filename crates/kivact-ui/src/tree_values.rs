//! Role: Read-only tree-row facts available to theme recipes and marker slots.
// xtask-allow(state-fields): Actual interaction output and ancestry projection, without independent lifecycle state.
/// The mounted row's interaction state and structural position in its visible tree.
#[derive(Clone, PartialEq)]
pub struct TreeRowRecipeState {
    pub item: kivact_interaction::TreeItemState,
    /// Whether each ancestor is the last visible sibling at its level.
    pub ancestor_last: Vec<bool>,
    pub is_last: bool,
    pub has_branches: bool,
    /// The caller supplied marker content instead of the standard glyph.
    pub custom_marker: bool,
}
impl std::ops::Deref for TreeRowRecipeState {
    type Target = kivact_interaction::TreeItemState;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
