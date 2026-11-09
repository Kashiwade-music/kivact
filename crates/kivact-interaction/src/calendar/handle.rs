//! Role: The `CalendarGridApi` handle: grid, rows, paging buttons and day cells.

use super::data::{CalendarDayState, CalendarSpan, DaySelection};
use super::policy::{CalendarOptions, CalendarSelection};
use super::state;
use super::state::{Choice, State};
use crate::roving::Member;
use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Owned, Owner, Part, PartState, part, raise, require_setup};
use kivact_core::i18n::{CalendarDate, CalendarMonth, Weekday};
use kivact_core::state::{IntoValue, Value, memo};
use kivact_core::view::{Handled, KeyEvent, Role};
use std::rc::Rc;

/// A Copy calendar handle over a caller-owned month and selection.
///
/// See the [module](super) for what the keys do. The parts are [`grid`](Self::grid),
/// [`row`](Self::row), [`cell`](Self::cell) and the four paging buttons
/// [`previous_month`](Self::previous_month), [`next_month`](Self::next_month),
/// [`previous_year`](Self::previous_year) and [`next_year`](Self::next_year).
#[derive(Clone, Copy)]
pub struct CalendarGridApi {
    state: Owned<Rc<State>>,
}

/// Create a setup-owned calendar grid without rendering any elements.
///
/// `month` is the first shown month and the caller's, the sole month
/// authority: paging writes it. Setup only.
#[track_caller]
pub fn use_calendar_grid(
    month: impl kivact_core::state::IntoBinding<CalendarMonth>,
    selection: CalendarSelection,
    policy: CalendarOptions,
) -> CalendarGridApi {
    require_setup("use_calendar_grid");
    if policy.select_on_move && matches!(selection, CalendarSelection::Range(_)) {
        raise(
            ErrorKind::Message,
            "use_calendar_grid: CalendarOptions::select_on_move applies to a single day",
        );
    }
    CalendarGridApi {
        state: state::create(month.into_binding(), selection, policy),
    }
}

impl CalendarGridApi {
    /// Polite feedback emitted by accepted selection transitions.
    pub fn selection_status(self) -> Value<String> {
        self.state().selection_status.into_value()
    }

    /// Whether the range owner is waiting for its second endpoint.
    pub fn is_picking_range(self) -> Value<bool> {
        let choice = self.state().choice;
        memo(move || match choice {
            Choice::Range { pick, .. } => {
                matches!(pick.get(), super::machine::RangePick::Picking { .. })
            }
            Choice::Single(_) => false,
        })
    }

    /// Drawing state of the mounted calendar grid.
    pub fn grid_state(self) -> Value<PartState> {
        let observed = self.state().grid_state;
        memo(move || observed.get().map(|state| state.get()).unwrap_or_default())
    }
    /// Drawing state and logical selection marks of a calendar day.
    pub fn cell_state(self, day: CalendarDate) -> Value<CalendarDayState> {
        let shared = self.state();
        memo(move || {
            shared
                .cell_states
                .with(|states| states.get(&day).copied())
                .map(|state| state.get())
                .unwrap_or_else(|| {
                    let (selection, span) = marks(shared.choice, day);
                    CalendarDayState {
                        input: PartState::default(),
                        selection,
                        span,
                        disabled: shared.disabled(day),
                    }
                })
        })
    }

    fn state(self) -> Rc<State> {
        self.state.get()
    }

    /// Preview a range endpoint during pointer exploration. It never writes
    /// the caller's selection; leaving a cell clears the preview.
    pub fn preview(self, day: Option<CalendarDate>) {
        if let Choice::Range { pick, .. } = self.state().choice {
            pick.send(super::machine::Event::Preview(day));
        }
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
        let multiple = matches!(self.state().choice, Choice::Range { .. });
        let input = part(|input| input);
        let part_state = input.state();
        self.state().grid_state.set(Some(part_state));
        Part::new(part_state, move |host| {
            host.part(input).role(Role::Grid).multiselectable(multiple)
        })
    }

    /// One week of a shown month: a `Row`.
    #[track_caller]
    pub fn row(self) -> Part<PartState> {
        let input = part(|input| input);
        let part_state = input.state();
        Part::new(part_state, move |host| host.part(input).role(Role::Row))
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
        let input = part(|input| input);
        let part_state = input.state();
        Part::new(part_state, move |host| {
            host.part(input)
                .role(Role::Button)
                .focusable(true)
                .on_press(move || month.set(month.get_untracked().add_months(months)))
        })
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
        let state = self.state();
        let disabled = {
            let state = state.clone();
            memo(move || state.disabled(day))
        };
        let choice = state.choice;
        let marks = memo(move || marks(choice, day));
        let input = part(move |input| {
            let (selection, span) = marks.get();
            CalendarDayState {
                input,
                selection,
                span,
                disabled: disabled.get(),
            }
        });
        let part_state = input.state();
        let observed = state.cell_states;
        observed.update(|states| {
            states.insert(day, part_state);
        });
        kivact_core::state::on_cleanup(move || {
            if observed.try_get_untracked().is_some() {
                observed.update(|states| {
                    if states.get(&day) == Some(&part_state) {
                        states.remove(&day);
                    }
                });
            }
        });
        Part::new(part_state, move |host| {
            let attachment = host
                .part(input)
                .role(Role::GridCell)
                .name(day.to_string())
                .selected(memo(move || marks.get().0 != DaySelection::None))
                .disabled(disabled);
            let pointers = state.clone();
            let preview = state.clone();
            let attachment =
                attachment
                    .on_press(move || pointers.pick(day))
                    .on_pointer(move |event| {
                        if let Choice::Range { pick, .. } = preview.choice
                            && event.phase() == kivact_core::view::PointerPhase::Move
                        {
                            pick.send(super::machine::Event::Preview(Some(day)));
                        }
                        Handled::No
                    });
            if day.month_of() != month {
                return attachment;
            }
            let active = state.active;
            let is_active = memo(move || active.get() == Some(day));
            let keys = state.clone();
            attachment
                .focusable(is_active)
                .on_key(move |event: KeyEvent| -> Handled { keys.key(day, &event) })
                .on_attach(move |mount| {
                    mount_cell(state.clone(), day, mount);
                })
        })
    }
}

/// Whether the day is selected, drawn solid, and where it lies in a range.
fn marks(choice: Choice, day: CalendarDate) -> (DaySelection, CalendarSpan) {
    match choice {
        Choice::Single(value) => {
            let chosen = value.with(|value| *value == Some(day));
            (
                if chosen {
                    DaySelection::Edge
                } else {
                    DaySelection::None
                },
                CalendarSpan::None,
            )
        }
        Choice::Range { value, pick } => {
            let range = match pick.get() {
                super::machine::RangePick::Picking {
                    anchor, preview, ..
                } => Some(kivact_core::i18n::DateRange::new(
                    anchor,
                    preview.unwrap_or(anchor),
                )),
                _ => value.get(),
            };
            match range {
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
                    let selection = if day == range.start() || day == range.end() {
                        DaySelection::Edge
                    } else {
                        DaySelection::Inside
                    };
                    (selection, span)
                }
                _ => (DaySelection::None, CalendarSpan::None),
            }
        }
    }
}

fn mount_cell(state: Rc<State>, day: CalendarDate, mount: kivact_core::custom::PartMount) {
    let owner = Owner::current();
    if Owner::root() != state.root {
        owner.report(
            ErrorKind::WrongApp,
            "CalendarGridApi cell belongs to another App",
        );
        return;
    }
    if state.roving.contains(&day) {
        owner.report(
            ErrorKind::Message,
            "CalendarGridApi: a cell for this day is already mounted",
        );
        return;
    }
    let node = mount.node;
    state.roving.insert(
        day,
        Member {
            node,
            eligible: mount.eligible,
            disabled: mount.disabled,
            hidden: mount.hidden,
        },
    );
    let mounted = node.mounted_node();
    let weak = Rc::downgrade(&state);
    owner.on_cleanup(move || {
        let Some(state) = weak.upgrade() else {
            return;
        };
        let focused = mounted.is_some() && state.owner.focused_node() == mounted;
        if state.roving.remove(&day, node) && focused {
            state.roving.restore_from.set(mounted);
        }
    });
}
