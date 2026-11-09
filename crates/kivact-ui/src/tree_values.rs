//! Role: Read-only tree-row facts available to theme recipes and marker slots.
/// The mounted row's interaction state and structural position in its visible tree.
#[derive(Clone, PartialEq)]
pub struct TreeRowRecipeState {
    pub item: kivact_interaction::TreeItemState,
    /// Whether each ancestor is the last visible sibling at its level.
    pub ancestor_last: Vec<bool>,
    // xtask-allow(state-fields): passive record - is last
    pub is_last: bool,
    // xtask-allow(state-fields): passive record - has branches
    pub has_branches: bool,
    // xtask-allow(state-fields): passive record - The caller supplied marker content instead of the standard glyph
    /// The caller supplied marker content instead of the standard glyph.
    pub custom_marker: bool,
}
impl std::ops::Deref for TreeRowRecipeState {
    type Target = kivact_interaction::TreeItemState;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
