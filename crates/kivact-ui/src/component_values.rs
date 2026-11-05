//! Role: Define passive component recipe projections.
// xtask-allow(state-fields): Read-only recipe projection of the caller binding and actual interaction configuration.
/// Card semantics supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardState {
    pub selected: Option<bool>,
    pub input: Option<kivact_core::custom::PartState>,
    pub disabled: bool,
    pub interactive: bool,
}
