//! Role: Describe passive semantic projections supplied to component recipes.
use crate::values::ControlSize;
use kivact_core::view::ReadingDirection;
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
    pub disabled: bool,
    pub interactive: bool,
    pub has_media: bool,
    pub has_footer: bool,
}

/// The semantic state passed to every empty-state recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmptyStateState {
    pub heading_level: u8,
}

/// Input group size and reading direction passed to its recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupState {
    pub size: ControlSize,
    pub direction: ReadingDirection,
}
// xtask-allow(state-fields): Read-only projection of member position and NodeRef focus-within, never independent behavior state.
/// A member's position, inherited size, and actual focus-within observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupMemberState {
    pub group: InputGroupState,
    pub first: bool,
    pub last: bool,
    pub focused: bool,
}
/// Semantic heading level passed to its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadingState {
    /// Heading level, clamped to 1 through 6.
    pub level: u8,
}
/// The direction of change, independent of whether that change is desirable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Trend {
    /// The statistic has increased.
    Up,
    /// The statistic has decreased.
    Down,
    /// The statistic is steady or has no stated direction.
    #[default]
    Neutral,
}
kivact_core::state::constant_value!(Trend);
/// Semantic state shared by all statistic recipes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatState {
    /// Direction supplied by the application; it does not imply a success or danger tone.
    pub trend: Trend,
}
