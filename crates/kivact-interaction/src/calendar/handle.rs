//! Role: The `CalendarGrid` handle: grid, rows, paging buttons and day cells with their state.

use super::policy::{CalendarPolicy, CalendarSelection};
use super::state;
use super::state::State;
use crate::roving::Member;
use kivact_core::diagnostics::ErrorKind;
use kivact_core::i18n::{CalendarDate, CalendarMonth, Weekday};
use kivact_core::primitive::{NodeRefExt, Owned, Owner, Part, PartState, part, require_setup};
use kivact_core::view::{Handled, KeyEvent, Role};
use kivact_core::{Binding, Value, memo};
use std::rc::Rc;

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

/// The reactive drawing state of a [`cell`](CalendarGrid::cell).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarDayState {
    pub input: PartState,
    /// The day is chosen or inside the chosen range; it is published as
    /// selected.
    pub selected: bool,
    /// The chosen day, or an end of the chosen range: what is drawn solid.
    pub edge: bool,
    /// Where the day lies in a range of several days.
    pub span: CalendarSpan,
    /// The policy refuses the day.
    pub disabled: bool,
}

/// A Copy calendar handle over a caller-owned month and selection.
///
/// See the [module](super) for what the keys do. The parts are [`grid`](Self::grid),
/// [`row`](Self::row), [`cell`](Self::cell) and the four paging buttons
/// [`previous_month`](Self::previous_month), [`next_month`](Self::next_month),
/// [`previous_year`](Self::previous_year) and [`next_year`](Self::next_year).
#[derive(Clone, Copy)]
pub struct CalendarGrid {
    state: Owned<State>,
}

/// Create a setup-owned calendar grid without rendering any elements.
///
/// `month` is the first shown month and the caller's, the sole month
/// authority: paging writes it. Setup only.
#[track_caller]
pub fn use_calendar_grid(
    month: impl Into<Binding<CalendarMonth>>,
    selection: CalendarSelection,
    policy: CalendarPolicy,
) -> CalendarGrid {
    require_setup("use_calendar_grid");
    CalendarGrid {
        state: state::create(month.into(), selection, policy),
    }
}

impl CalendarGrid {
    fn state(self) -> Rc<State> {
        self.state.get()
    }

    /// The first day of the week the grid lays out.
    pub fn week_start(self) -> Value<Weekday> {
        self.state().first_day
    }

    /// Focus the day that holds the tab stop, if it is mounted: the focus a
    /// calendar takes when it opens. Call it from `on_mount`. It returns
    /// whether a day took the focus.
    #[track_caller]
    pub fn focus_active(self) -> bool {
        self.state().focus_active()
    }

    /// The container of one shown month: a `Grid` that is multiselectable for
    /// a range. Its name and drawing stay the caller's.
    #[track_caller]
    pub fn grid(self) -> Part<PartState> {
        let multiple = matches!(self.state().selection, CalendarSelection::Range(_));
        part(|input| input)
            .role(Role::Grid)
            .multiselectable(multiple)
    }

    /// One week of a shown month: a `Row`.
    #[track_caller]
    pub fn row(self) -> Part<PartState> {
        part(|input| input).role(Role::Row)
    }

    /// A button that moves the shown months back by one month.
    #[track_caller]
    pub fn previous_month(self) -> Part<PartState> {
        self.page(-1)
    }

    /// A button that moves the shown months forward by one month.
    #[track_caller]
    pub fn next_month(self) -> Part<PartState> {
        self.page(1)
    }

    /// A button that moves the shown months back by one year.
    #[track_caller]
    pub fn previous_year(self) -> Part<PartState> {
        self.page(-12)
    }

    /// A button that moves the shown months forward by one year.
    #[track_caller]
    pub fn next_year(self) -> Part<PartState> {
        self.page(12)
    }

    fn page(self, months: i32) -> Part<PartState> {
        let month = self.state().month;
        part(|input| input)
            .role(Role::Button)
            .focusable(true)
            .on_press(move || month.set(month.get_untracked().add_months(months)))
    }

    /// One day of the shown month `month`, which the caller draws as a node
    /// named by the day's ISO date (`2026-10-03`).
    ///
    /// A day inside `month` is a `GridCell` that can take the tab stop. A day
    /// outside it (a neighbour drawn in the first or last week) is a clickable
    /// `GridCell` that cannot take focus. Both are published as selected and
    /// disabled as the selection and the policy say.
    #[track_caller]
    pub fn cell(self, month: CalendarMonth, day: CalendarDate) -> Part<CalendarDayState> {
        let at = std::panic::Location::caller();
        let state = self.state();
        let disabled = {
            let policy = state.policy.disabled.clone();
            memo(move || policy(day))
        };
        let selection = state.selection;
        let marks = memo(move || marks(selection, day));
        let attachment = part(move |input| {
            let (selected, edge, span) = marks.get();
            CalendarDayState {
                input,
                selected,
                edge,
                span,
                disabled: disabled.get(),
            }
        })
        .role(Role::GridCell)
        .name(day.to_string())
        .selected(memo(move || marks.get().0))
        .disabled(disabled);
        let pointers = state.clone();
        let attachment = attachment.on_press(move || pointers.pick(day));
        if day.month_of() != month {
            return attachment;
        }
        let active = state.active;
        let is_active = memo(move || active.get() == Some(day));
        let keys = state.clone();
        attachment
            .focusable(is_active)
            .on_key(move |event: KeyEvent| -> Handled { keys.key(day, &event) })
            .on_bind(move |mount| {
                let owner = Owner::current("CalendarGrid::cell");
                if Owner::root("CalendarGrid::cell") != state.root {
                    owner.report(
                        ErrorKind::WrongApp,
                        "CalendarGrid cell belongs to another App",
                        at,
                    );
                    return;
                }
                if state.roving.members.borrow().contains_key(&day) {
                    owner.report(
                        ErrorKind::Message,
                        "CalendarGrid: a cell for this day is already mounted",
                        at,
                    );
                    return;
                }
                let node = mount.node;
                state.roving.members.borrow_mut().insert(
                    day,
                    Member {
                        node,
                        eligible: mount.eligible,
                        disabled: mount.disabled,
                        hidden: mount.hidden,
                    },
                );
                state.revision.update(|revision| *revision += 1);
                let mounted = node.mounted_node();
                let weak = Rc::downgrade(&state);
                owner.on_cleanup(move || {
                    let Some(state) = weak.upgrade() else {
                        return;
                    };
                    if !state
                        .roving
                        .members
                        .borrow()
                        .get(&day)
                        .is_some_and(|entry| entry.node.same_ref(node))
                    {
                        return;
                    }
                    let focused = mounted.is_some() && state.owner.focused_node() == mounted;
                    state.roving.members.borrow_mut().remove(&day);
                    if state.alive.try_get().is_some() {
                        if focused {
                            state.roving.restore_from.set(mounted);
                        }
                        state.revision.update(|revision| *revision += 1);
                    }
                });
            })
    }
}

/// Whether the day is selected, drawn solid, and where it lies in a range.
fn marks(selection: CalendarSelection, day: CalendarDate) -> (bool, bool, CalendarSpan) {
    match selection {
        CalendarSelection::Single(value) => {
            let chosen = value.with(|value| *value == Some(day));
            (chosen, chosen, CalendarSpan::None)
        }
        CalendarSelection::Range(value) => value.with(|range| match range {
            Some(range) if range.contains(day) => {
                let span = if range.start() == range.end() {
                    CalendarSpan::None
                } else if day == range.start() {
                    CalendarSpan::Start
                } else if day == range.end() {
                    CalendarSpan::End
                } else {
                    CalendarSpan::Middle
                };
                (true, day == range.start() || day == range.end(), span)
            }
            _ => (false, false, CalendarSpan::None),
        }),
    }
}
