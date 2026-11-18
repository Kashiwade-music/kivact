//! Role: The interaction states a style names, and sets of them.

/// A state a style can name (`.hover(..)`, `.active(..)`, ...). The host
/// evaluates the eight of [`StyleState::HOST`]; the core evaluates
/// `Disabled` and `Selected`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StyleState {
    /// The pointer is over the node or a node below it (CSS `:hover`).
    Hover,
    /// The node is pressed while hovered (CSS `:active`).
    Active,
    /// The node has the focus (CSS `:focus`).
    Focus,
    /// The node has the focus and the host shows a focus indicator (CSS `:focus-visible`).
    FocusVisible,
    /// The nearest group ancestor is hovered.
    GroupHover,
    /// The focus is on the nearest group ancestor or below it.
    GroupFocus,
    /// The node is disabled.
    Disabled,
    /// The node is selected.
    Selected,
    /// The paint the node enters from (CSS `@starting-style`).
    Enter,
    /// The paint a removed node leaves to.
    Exit,
}

impl StyleState {
    /// Every state, in declaration order.
    pub const ALL: [StyleState; 10] = [
        StyleState::Hover,
        StyleState::Active,
        StyleState::Focus,
        StyleState::FocusVisible,
        StyleState::GroupHover,
        StyleState::GroupFocus,
        StyleState::Disabled,
        StyleState::Selected,
        StyleState::Enter,
        StyleState::Exit,
    ];

    /// The host-evaluated states in the order their deltas stack, the
    /// lowest first: a later one overrides what it sets.
    pub const HOST: [StyleState; 8] = [
        StyleState::Focus,
        StyleState::FocusVisible,
        StyleState::GroupFocus,
        StyleState::GroupHover,
        StyleState::Hover,
        StyleState::Active,
        StyleState::Enter,
        StyleState::Exit,
    ];

    /// The index of a host-evaluated state in [`StyleState::HOST`].
    pub const fn host_slot(self) -> Option<usize> {
        match self {
            StyleState::Focus => Some(0),
            StyleState::FocusVisible => Some(1),
            StyleState::GroupFocus => Some(2),
            StyleState::GroupHover => Some(3),
            StyleState::Hover => Some(4),
            StyleState::Active => Some(5),
            StyleState::Enter => Some(6),
            StyleState::Exit => Some(7),
            StyleState::Disabled | StyleState::Selected => None,
        }
    }

    /// The name of the style method (`"hover"`, `"active"`, `"focus_visible"`).
    pub const fn name(self) -> &'static str {
        match self {
            StyleState::Hover => "hover",
            StyleState::Active => "active",
            StyleState::Focus => "focus",
            StyleState::FocusVisible => "focus_visible",
            StyleState::GroupHover => "group_hover",
            StyleState::GroupFocus => "group_focus",
            StyleState::Disabled => "disabled",
            StyleState::Selected => "selected",
            StyleState::Enter => "enter",
            StyleState::Exit => "exit",
        }
    }

    const fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// A set of [`StyleState`]s.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct StateSet(u16);

impl StateSet {
    /// No state.
    pub const EMPTY: StateSet = StateSet(0);

    /// This set with `state` added.
    pub const fn with(self, state: StyleState) -> StateSet {
        StateSet(self.0 | state.bit())
    }

    /// Add `state`.
    pub fn insert(&mut self, state: StyleState) {
        self.0 |= state.bit();
    }

    /// Whether `state` is in the set.
    pub const fn contains(self, state: StyleState) -> bool {
        self.0 & state.bit() != 0
    }

    /// Whether no state is in the set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The states in both sets.
    pub const fn intersection(self, other: StateSet) -> StateSet {
        StateSet(self.0 & other.0)
    }

    /// The states in either set.
    pub const fn union(self, other: StateSet) -> StateSet {
        StateSet(self.0 | other.0)
    }

    /// The states in the set, in declaration order.
    pub fn iter(self) -> impl Iterator<Item = StyleState> {
        StyleState::ALL
            .into_iter()
            .filter(move |state| self.contains(*state))
    }
}
