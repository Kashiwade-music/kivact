//! Role: Describe passive semantic projections supplied to component recipes.
use crate::values::ControlSize;
use kivact_core::view::ReadingDirection;
use kivact_interaction::Tone;

/// Alert meaning and disclosure state supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertState {
    pub tone: Tone,
    pub expansion: Expansion,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub dismissible: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_icon: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_actions: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_description: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_content: bool,
}

/// Card semantics supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardState {
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub input: Option<kivact_core::custom::PartState>,
    pub selection: Selectable,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub disabled: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub interactive: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_media: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_footer: bool,
}

/// Whether a part can expand and, if so, whether it is expanded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expansion {
    /// The part has no expandable content.
    NotExpandable,
    /// The part has content and shows it closed.
    Collapsed,
    /// The part shows its content.
    Expanded,
}

impl Expansion {
    /// The expansion of a part which may have content, open or closed.
    pub fn new(expandable: bool, open: bool) -> Self {
        match (expandable, open) {
            (false, _) => Self::NotExpandable,
            (true, false) => Self::Collapsed,
            (true, true) => Self::Expanded,
        }
    }
}

/// Whether a part can be selected and, if so, whether it is selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selectable {
    /// The part has no selected state.
    NotSelectable,
    /// The part can be selected and is not.
    Unselected,
    /// The part is selected.
    Selected,
}

impl Selectable {
    /// The selection of a part which may be selectable, selected or not.
    pub fn new(selectable: bool, selected: bool) -> Self {
        match (selectable, selected) {
            (false, _) => Self::NotSelectable,
            (true, false) => Self::Unselected,
            (true, true) => Self::Selected,
        }
    }
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
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub focused: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub invalid: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_start: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_end: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_block_start: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub has_block_end: bool,
    pub size: ControlSize,
    pub direction: ReadingDirection,
}
/// A member's position, inherited size, and actual focus-within observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupMemberState {
    pub kind: InputGroupMemberKind,
    pub group: InputGroupState,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub first: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
    pub last: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the component's state for recipes, written only by its render function
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
