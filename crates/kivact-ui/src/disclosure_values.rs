//! Role: Expose actual disclosure input and passive accordion position to theme recipes.
/// A panel's fixed position within its accordion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosureGroupPosition {
    /// Zero-based panel index.
    pub index: usize,
    /// Number of panels in the group.
    pub count: usize,
}
// xtask-allow(state-fields): Passive recipe projection of actual interaction state and immutable group position; owns no lifecycle or selection authority.
/// Actual disclosure state with optional group geometry for theme decoration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosureRecipeState {
    pub input: kivact_core::custom::PartState,
    pub open: bool,
    pub group: Option<DisclosureGroupPosition>,
}
impl DisclosureRecipeState {
    pub(crate) fn from_part(
        state: kivact_interaction::DisclosureState,
        group: Option<DisclosureGroupPosition>,
    ) -> Self {
        Self {
            input: state.input,
            open: state.open,
            group,
        }
    }
}
