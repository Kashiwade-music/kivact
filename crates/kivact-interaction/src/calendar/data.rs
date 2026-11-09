//! Role: Drawing contracts shared by calendar cells and their registry.

use kivact_core::custom::PartState;

/// How a day lies in the selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CalendarSpan {
    /// The first day of a range of several days.
    Start,
    /// A day between the ends of a range of several days.
    Middle,
    /// The last day of a range of several days.
    End,
    /// Not in a range of several days (a chosen day or a one-day range, if
    /// [`CalendarDayState::selection`] is [`DaySelection::Edge`]).
    #[default]
    None,
}

/// Where a day lies in the selection: published as `selected` unless
/// [`DaySelection::None`], and drawn solid when it is [`DaySelection::Edge`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DaySelection {
    /// The day is not in the selection.
    #[default]
    None,
    /// The day is inside the chosen range of several days.
    Inside,
    /// The chosen day, or an end of the chosen range: what is drawn solid.
    Edge,
}

/// The reactive drawing state of a [`cell`](super::CalendarGridApi::cell).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarDayState {
    pub input: PartState,
    /// How the day lies in the selection.
    pub selection: DaySelection,
    /// Where the day lies in a range of several days.
    pub span: CalendarSpan,
    // xtask-allow(state-fields): projection: readonly snapshot of the calendar policy for recipes, written only by the cell part
    /// The policy refuses the day.
    pub disabled: bool,
}
