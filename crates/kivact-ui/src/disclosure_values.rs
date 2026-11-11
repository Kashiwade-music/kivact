//! Role: Expose actual disclosure input and passive accordion position to theme recipes.
/// A panel's fixed position within its accordion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosureGroupPosition {
    /// Zero-based panel index.
    pub index: usize,
    /// Number of panels in the group.
    pub count: usize,
}
/// Actual disclosure state with optional group geometry for theme decoration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosureRecipeState {
    pub input: kivact_core::custom::PartState,
    // xtask-allow(state-fields): independent: copied from the attached part's DisclosureState::open by DisclosureRecipeState::from_part; standalone panels and accordion members (group Some) both open and close
    pub open: bool,
    // xtask-allow(state-fields): option: the panel's place in an accordion; absent for a standalone disclosure
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
