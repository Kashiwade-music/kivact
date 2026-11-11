//! Role: Describe passive semantic projections supplied to component recipes.
use crate::values::ControlSize;
use kivact_core::view::ReadingDirection;
use kivact_interaction::Tone;

/// Alert meaning and disclosure state supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertState {
    pub tone: Tone,
    // xtask-allow(state-fields): independent: Alert::content fills the content slot (Some) and Alert::collapsible with the open binding choose its Expansion; it combines with any icon, actions, description or dismiss button
    /// The content slot ([`Alert::content`](super::Alert::content)): `None`
    /// without content, otherwise whether it can collapse and is open.
    pub content: Option<Expansion>,
    // xtask-allow(state-fields): independent: Alert::dismissible shows the dismiss button, with or without content, icon or actions
    pub dismissible: bool,
    // xtask-allow(state-fields): independent: Alert::icon fills the icon slot of a dismissible or fixed alert alike
    pub has_icon: bool,
    // xtask-allow(state-fields): independent: Alert::actions fills the action slot, beside or without a dismiss button
    pub has_actions: bool,
    // xtask-allow(state-fields): independent: Alert::description adds the description line, with or without content below it
    pub has_description: bool,
}

impl AlertState {
    /// Whether the alert has a content slot ([`Alert::content`](super::Alert::content)),
    /// collapsible or not.
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }
    /// Whether the content can collapse and, if so, whether it is open;
    /// [`NotExpandable`](Expansion::NotExpandable) without content or when
    /// it cannot collapse.
    pub fn expansion(&self) -> Expansion {
        self.content.unwrap_or(Expansion::NotExpandable)
    }
}

/// Card semantics supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardState {
    /// What activating the card does; `match` it, or read
    /// [`input`](Self::input), [`is_interactive`](Self::is_interactive) and
    /// [`selection`](Self::selection).
    pub action: CardAction,
    // xtask-allow(state-fields): independent: the caller's Card::disabled value applies to any action
    pub disabled: bool,
    // xtask-allow(state-fields): independent: media and footer are separate optional slots; all four combinations render
    pub has_media: bool,
    // xtask-allow(state-fields): independent: media and footer are separate optional slots; all four combinations render
    pub has_footer: bool,
}

impl CardState {
    /// The actual input state of the card's button or check box part;
    /// `None` for a plain card.
    pub fn input(&self) -> Option<kivact_core::custom::PartState> {
        match self.action {
            CardAction::Plain => None,
            CardAction::Press { input } | CardAction::Select { input, .. } => Some(input),
        }
    }
    /// Whether the card can be pressed or selected.
    pub fn is_interactive(&self) -> bool {
        !matches!(self.action, CardAction::Plain)
    }
    /// Whether the card can be selected and, if so, whether it is.
    pub fn selection(&self) -> Selectable {
        match self.action {
            CardAction::Select { selected, .. } => Selectable::new(true, selected),
            CardAction::Plain | CardAction::Press { .. } => Selectable::NotSelectable,
        }
    }
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

/// What activating a card does, with the actual input of the part that does it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardAction {
    /// A plain surface; nothing activates it.
    Plain,
    /// Pressing activates it ([`Card::on_press`](super::Card::on_press)).
    Press {
        /// The actual input state of the card's button part.
        input: kivact_core::custom::PartState,
    },
    /// It is a check box ([`Card::selected`](super::Card::selected)).
    Select {
        /// The actual input state of the card's check box part.
        input: kivact_core::custom::PartState,
        /// Whether the card is selected.
        selected: bool,
    },
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
    // xtask-allow(state-fields): independent: focus is within the group (focus_within of its root); invalid and any adornment combine with it
    pub focused: bool,
    // xtask-allow(state-fields): independent: the group's .invalid(..) (accessible_invalid); a group is invalid whether focused or not
    pub invalid: bool,
    // xtask-allow(state-fields): independent: InputGroup::start fills the inline start adornment, with or without an end one
    pub has_start: bool,
    // xtask-allow(state-fields): independent: InputGroup::end fills the inline end adornment, with or without a start one
    pub has_end: bool,
    // xtask-allow(state-fields): independent: InputGroup::block_start adds a row above the controls, beside any inline adornments
    pub has_block_start: bool,
    // xtask-allow(state-fields): independent: InputGroup::block_end adds a row below the controls, beside any inline adornments
    pub has_block_end: bool,
    pub size: ControlSize,
    pub direction: ReadingDirection,
}
/// A member's position, inherited size, and actual focus-within observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupMemberState {
    pub kind: InputGroupMemberKind,
    pub group: InputGroupState,
    // xtask-allow(state-fields): independent: index == 0 in GroupRender::member; a single member is both first and last
    pub first: bool,
    // xtask-allow(state-fields): independent: index + 1 == count in GroupRender::member; the middle member of three is neither
    pub last: bool,
    // xtask-allow(state-fields): independent: focus is within this member (focus_within of its node), at any position in the group
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
