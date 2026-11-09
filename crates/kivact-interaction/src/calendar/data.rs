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
    /// [`CalendarDayState::edge`] is true).
    #[default]
    None,
}

/// The reactive drawing state of a [`cell`](super::CalendarGridApi::cell).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarDayState {
    pub input: PartState,
    // xtask-allow(state-fields): passive record - The day is chosen or inside the chosen range;
    /// The day is chosen or inside the chosen range; it is published as
    /// selected.
    pub selected: bool,
    // xtask-allow(state-fields): passive record - The chosen day, or an end of the chosen range: what is drawn solid
    /// The chosen day, or an end of the chosen range: what is drawn solid.
    pub edge: bool,
    /// Where the day lies in a range of several days.
    pub span: CalendarSpan,
    // xtask-allow(state-fields): passive record - The policy refuses the day
    /// The policy refuses the day.
    pub disabled: bool,
}
