//! Role: Read-only tree-row facts available to theme recipes and marker slots.
/// The mounted row's interaction state and structural position in its visible tree.
#[derive(Clone, PartialEq)]
pub struct TreeRowRecipeState {
    pub item: kivact_interaction::TreeItemState,
    /// Whether each ancestor is the last visible sibling at its level.
    pub ancestor_last: Vec<bool>,
    // xtask-allow(state-fields): independent: the row is the last visible sibling at its level (TreeRow::is_last); last rows may be branches or leaves
    pub is_last: bool,
    // xtask-allow(state-fields): independent: some visible row of the whole tree has children; every row, last or not, reads the same value
    pub has_branches: bool,
    // xtask-allow(state-fields): independent: Tree::marker_content was given; it replaces the glyph on last and inner rows alike
    /// The caller supplied marker content instead of the standard glyph.
    pub custom_marker: bool,
}
impl std::ops::Deref for TreeRowRecipeState {
    type Target = kivact_interaction::TreeItemState;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
