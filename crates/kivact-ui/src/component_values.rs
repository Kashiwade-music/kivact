//! Role: Describe passive semantic projections supplied to component recipes.
use crate::values::ControlSize;
use kivact_core::view::ReadingDirection;
use kivact_interaction::Tone;

/// Alert meaning and disclosure state supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertState {
    pub tone: Tone,
    // xtask-allow(state-fields): passive record - expanded
    pub expanded: Option<bool>,
    // xtask-allow(state-fields): passive record - dismissible
    pub dismissible: bool,
    // xtask-allow(state-fields): passive record - has icon
    pub has_icon: bool,
    // xtask-allow(state-fields): passive record - has actions
    pub has_actions: bool,
    // xtask-allow(state-fields): passive record - has description
    pub has_description: bool,
    // xtask-allow(state-fields): passive record - has content
    pub has_content: bool,
}

/// Card semantics supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardState {
    // xtask-allow(state-fields): passive record - input
    pub input: Option<kivact_core::custom::PartState>,
    // xtask-allow(state-fields): passive record - selected
    pub selected: Option<bool>,
    // xtask-allow(state-fields): passive record - disabled
    pub disabled: bool,
    // xtask-allow(state-fields): passive record - interactive
    pub interactive: bool,
    // xtask-allow(state-fields): passive record - has media
    pub has_media: bool,
    // xtask-allow(state-fields): passive record - has footer
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
    pub input: kivact_core::custom::PartState,
    // xtask-allow(state-fields): passive record - focused
    pub focused: bool,
    // xtask-allow(state-fields): passive record - invalid
    pub invalid: bool,
    // xtask-allow(state-fields): passive record - has start
    pub has_start: bool,
    // xtask-allow(state-fields): passive record - has end
    pub has_end: bool,
    // xtask-allow(state-fields): passive record - has block start
    pub has_block_start: bool,
    // xtask-allow(state-fields): passive record - has block end
    pub has_block_end: bool,
    pub size: ControlSize,
    pub direction: ReadingDirection,
}
/// A member's position, inherited size, and actual focus-within observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupMemberState {
    pub kind: InputGroupMemberKind,
    pub group: InputGroupState,
    // xtask-allow(state-fields): passive record - first
    pub first: bool,
    // xtask-allow(state-fields): passive record - last
    pub last: bool,
    // xtask-allow(state-fields): passive record - focused
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
