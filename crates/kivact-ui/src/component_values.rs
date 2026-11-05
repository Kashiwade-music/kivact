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

// xtask-allow(state-fields): Read-only recipe projection of actual input and independent validation, focus-within and construction-slot facts; no mutable lifecycle authority.
/// Input group size and reading direction passed to its recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupState {
    pub input: kivact_core::custom::PartState,
    pub focused: bool,
    pub invalid: bool,
    pub has_start: bool,
    pub has_end: bool,
    pub has_block_start: bool,
    pub has_block_end: bool,
    pub size: ControlSize,
    pub direction: ReadingDirection,
}
// xtask-allow(state-fields): Read-only projection of member position and NodeRef focus-within, never independent behavior state.
/// A member's position, inherited size, and actual focus-within observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupMemberState {
    pub kind: InputGroupMemberKind,
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

/// The structural presentation of an alert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Default,
    Banner,
}
impl crate::variants::Variant for AlertVariant {}
/// The source slot of one actual inline input-group member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputGroupMemberKind {
    Start,
    Control,
    End,
}
impl Default for InputGroupState {
    fn default() -> Self {
        Self {
            input: Default::default(),
            focused: false,
            invalid: false,
            has_start: false,
            has_end: false,
            has_block_start: false,
            has_block_end: false,
            size: ControlSize::Medium,
            direction: ReadingDirection::Ltr,
        }
    }
}
