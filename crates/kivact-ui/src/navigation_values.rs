//! Role: Passive actual input facts for navigation link recipes.
/// Actual link input and independent description-slot availability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavigationLinkState {
    pub input: kivact_core::custom::PartState,
    pub has_description: bool,
}

/// Whether this actual navigation trigger's existing panel is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationTriggerState {
    pub open: bool,
}
