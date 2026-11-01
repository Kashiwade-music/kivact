//! Role: Calendar state shares the shown months, the selection and the day registry.

use super::policy::{CalendarPolicy, CalendarSelection};
use crate::policy::grid::GridMove;
use crate::roving::{Observation, Roving};
use kivact_core::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact_core::primitive::{Owned, Owner};
use kivact_core::view::{Handled, Key, KeyEvent};
use kivact_core::{Binding, Signal, Value, effect, memo, signal};
use std::rc::Rc;

pub(super) struct State {
    pub(super) owner: Owner,
    /// The App's root when the grid was created: a day of another App has
    /// another root.
    pub(super) root: Owner,
    /// Resolves while the grid's owner is alive and not being disposed.
    pub(super) alive: Owned<()>,
    pub(super) month: Binding<CalendarMonth>,
    pub(super) selection: CalendarSelection,
    pub(super) policy: CalendarPolicy,
    pub(super) first_day: Value<Weekday>,
    /// The first click of a range that waits for its second.
    pub(super) pending: Signal<Option<CalendarDate>>,
    /// The mounted days, the day the keyboard or pointer last moved to and
    /// the focus bookkeeping.
    pub(super) roving: Roving<CalendarDate>,
    /// The day that holds the tab stop.
    pub(super) active: Value<Option<CalendarDate>>,
    pub(super) revision: Signal<u64>,
}

pub(super) fn create(
    month: Binding<CalendarMonth>,
    selection: CalendarSelection,
    policy: CalendarPolicy,
) -> Owned<State> {
    let owner = Owner::current("use_calendar_grid");
    let root = Owner::root("use_calendar_grid");
    let alive = owner.store("CalendarGrid", Rc::new(()));
    let roving = Roving::<CalendarDate>::new();
    let requested = roving.requested;
    let revision = signal(0u64);
    let registry = roving.members.clone();
    let count = policy.months;
    let disabled = policy.disabled.clone();
    let initial = policy.initial_focus;
    let first_day = policy.week_start.unwrap_or_else(|| {
        let locale = kivact_core::i18n::locale();
        memo(move || locale.get().first_day_of_week())
    });
    let active = memo(move || {
        alive.try_get()?;
        revision.get();
        let first = month.get();
        let shown =
            |day: &CalendarDate| (0..count as i32).contains(&first.months_until(day.month_of()));
        let members = registry.borrow();
        let available = |day: &CalendarDate| {
            shown(day) && !disabled(*day) && members.get(day).is_some_and(|entry| entry.available())
        };
        let anchor = match selection {
            CalendarSelection::Single(value) => value.get(),
            CalendarSelection::Range(value) => value.get().map(DateRange::start),
        };
        [requested.get(), anchor, initial]
            .into_iter()
            .flatten()
            .find(|day| available(day))
            .or_else(|| {
                (0..count as i32)
                    .map(|m| first.add_months(m))
                    .flat_map(|month| {
                        (0..i32::from(month.days())).map(move |d| month.first_day().add_days(d))
                    })
                    .find(|day| available(day))
            })
    });
    let state = Rc::new(State {
        owner,
        root,
        alive,
        month,
        selection,
        policy,
        first_day,
        pending: signal(None),
        roving,
        active,
        revision,
    });
    let stored = owner.store("CalendarGrid", state.clone());
    effect(move || reconcile(&state));
    stored
}

/// Follow focus that reaches a day by other means, and give the focus of a
/// removed day to the tab stop.
fn reconcile(state: &State) {
    if state.alive.try_get().is_none() {
        return;
    }
    let active = state.active.get();
    let observation = Observation {
        current: state.owner.focused_node(),
        active: active.as_ref(),
        adopt: true,
        reject_refused: false,
    };
    state
        .roving
        .reconcile(observation, |day| !state.disabled(*day));
}

impl State {
    pub(super) fn disabled(&self, day: CalendarDate) -> bool {
        (self.policy.disabled)(day)
    }

    /// Whether `month` is one of the shown months.
    pub(super) fn shows(&self, month: CalendarMonth) -> bool {
        let offset = self.month.get_untracked().months_until(month);
        (0..self.policy.months as i32).contains(&offset)
    }

    /// The first enabled day from `from`, moving `step` days at a time.
    fn enabled_from(&self, from: CalendarDate, step: i32) -> Option<CalendarDate> {
        let mut day = from;
        for _ in 0..400 {
            if !self.disabled(day) {
                return Some(day);
            }
            day = day.add_days(step);
        }
        None
    }

    /// Focus the day that holds the tab stop.
    pub(super) fn focus_active(&self) -> bool {
        self.active.get_untracked().is_some_and(|day| {
            self.roving
                .focus(&day, |day| !self.disabled(*day))
                .is_some()
        })
    }

    /// Move the tab stop to `to`, showing its month and focusing its node.
    fn go_to(&self, to: CalendarDate) {
        self.roving.requested.set(Some(to));
        let target = to.month_of();
        if self.shows(target) {
            self.roving.focus(&to, |day| !self.disabled(*day));
            return;
        }
        // The days of the new months are mounted again; the focus the old
        // ones held goes to the tab stop, which is `to`.
        let shown = self.month.get_untracked();
        self.month.set(if target < shown {
            target
        } else {
            target.add_months(1 - self.policy.months as i32)
        });
    }

    /// Choose `day` as a press or Enter on it does.
    pub(super) fn pick(&self, day: CalendarDate) {
        if self.disabled(day) {
            return;
        }
        let done = match self.selection {
            CalendarSelection::Single(value) => {
                value.set(Some(day));
                true
            }
            CalendarSelection::Range(value) => match self.pending.get_untracked() {
                None => {
                    value.set(Some(DateRange::new(day, day)));
                    self.pending.set(Some(day));
                    false
                }
                Some(anchor) => {
                    value.set(Some(DateRange::new(anchor, day)));
                    self.pending.set(None);
                    true
                }
            },
        };
        self.roving.requested.set(Some(day));
        if !self.shows(day.month_of()) {
            self.month.set(day.month_of());
        }
        if done && let Some(commit) = self.policy.on_commit {
            commit.call(day);
        }
    }

    /// The arrow, Home, End and Page keys of a focused day.
    pub(super) fn key(&self, day: CalendarDate, event: &KeyEvent) -> Handled {
        let modifiers = event.modifiers;
        if modifiers.ctrl || modifiers.alt || modifiers.meta {
            return Handled::No;
        }
        let column = day.weekday().days_since(self.first_day.get_untracked()) as i32;
        let months = if modifiers.shift { 12 } else { 1 };
        let (from, step) = match (event.key, GridMove::from_key(event.key, modifiers)) {
            (Key::PageUp, _) => (day.add_months(-months), -1),
            (Key::PageDown, _) => (day.add_months(months), 1),
            (_, Some(GridMove::Left)) => (day.add_days(-1), -1),
            (_, Some(GridMove::Right)) => (day.add_days(1), 1),
            (_, Some(GridMove::Up)) => (day.add_days(-7), -7),
            (_, Some(GridMove::Down)) => (day.add_days(7), 7),
            (_, Some(GridMove::RowStart)) => (day.add_days(-column), 1),
            (_, Some(GridMove::RowEnd)) => (day.add_days(6 - column), -1),
            _ => return Handled::No,
        };
        if let Some(to) = self.enabled_from(from, step) {
            self.go_to(to);
            if self.policy.select_on_move
                && let CalendarSelection::Single(value) = self.selection
            {
                value.set(Some(to));
            }
        }
        Handled::Yes
    }
}
