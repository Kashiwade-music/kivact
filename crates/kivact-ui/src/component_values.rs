//! Role: Describe passive semantic projections supplied to component recipes.
use kivact_interaction::Tone;

// xtask-allow(state-fields): Read-only recipe projection of tone, disclosure binding and independent dismiss-control availability.
/// Alert meaning and disclosure state supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertState {
    pub tone: Tone,
    pub expanded: Option<bool>,
    pub dismissible: bool,
    pub has_icon: bool,
    pub has_actions: bool,
    pub has_description: bool,
}

// xtask-allow(state-fields): Read-only recipe projection of the caller binding and actual interaction configuration.
/// Card semantics supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardState {
    pub selected: Option<bool>,
    pub input: Option<kivact_core::custom::PartState>,
    pub disabled: bool,
    pub interactive: bool,
}
