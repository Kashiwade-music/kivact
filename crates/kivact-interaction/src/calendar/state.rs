//! Role: Calendar state shares the shown months, the selection and the day registry.

use super::machine::{self, RangePick};
use super::policy::{CalendarOptions, CalendarSelection};
use crate::policy::grid::GridMove;
use crate::roving::{Observation, Roving};
use kivact_core::custom::{Owned, Owner};
use kivact_core::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact_core::state::{Binding, MachineHandle, Signal, Value, effect, machine, memo, signal};
use kivact_core::view::{Handled, Key, KeyEvent};
use std::rc::Rc;

/// The caller's selection with what a range choice needs besides it.
#[derive(Clone, Copy)]
pub(super) enum Choice {
    Single(Binding<Option<CalendarDate>>),
    Range {
        value: Binding<Option<DateRange>>,
        pick: MachineHandle<machine::Machine>,
    },
}

pub(super) struct State {
    pub(super) chosen_message: Value<String>,
    pub(super) selection_status: Signal<String>,
    // xtask-allow(state-fields): Optional mounted-part drawing observation, independent of behavior phase.
    pub(super) grid_state: Signal<Option<Value<kivact_core::custom::PartState>>>,
    pub(super) cell_states:
        Signal<std::collections::HashMap<CalendarDate, Value<super::data::CalendarDayState>>>,
    pub(super) owner: Owner,
    /// The App's root when the grid was created: a day of another App has
    /// another root.
    pub(super) root: Owner,
    pub(super) month: Binding<CalendarMonth>,
    pub(super) choice: Choice,
    pub(super) policy: CalendarOptions,
    pub(super) first_day: Value<Weekday>,
    /// The mounted days, the day the keyboard or pointer last moved to and
    /// the focus bookkeeping.
    pub(super) roving: Roving<CalendarDate>,
    /// The day that holds the tab stop.
    // xtask-allow(state-fields): Pure projection of the canonical order, selection, and mounted registry.
    pub(super) active: Value<Option<CalendarDate>>,
}

pub(super) fn create(
    month: Binding<CalendarMonth>,
    selection: CalendarSelection,
    policy: CalendarOptions,
) -> Owned<Rc<State>> {
    let owner = Owner::current();
    let root = Owner::root();
    let roving = Roving::<CalendarDate>::new();
    let requested = roving.requested;
    let registry = roving.clone();
    let selection_status = signal(String::new());
    let chosen = kivact_core::i18n::message("calendar.chosen", "Date selected");
    let started = kivact_core::i18n::message("calendar.range_started", "Start date selected");
    let cancelled =
        kivact_core::i18n::message("calendar.range_cancelled", "Range selection cancelled");
    let choice = match selection {
        CalendarSelection::Single(value) => Choice::Single(value),
        CalendarSelection::Range(value) => {
            let pick = machine(
                RangePick::Idle,
                || machine::Machine,
                move |_, effect| match effect {
                    machine::Effect::Commit(range) => {
                        value.set(Some(range));
                        selection_status.set(format!(
                            "{}: {} - {}",
                            chosen.get_untracked(),
                            range.start(),
                            range.end()
                        ));
                    }
                    machine::Effect::Started(day) => {
                        selection_status.set(format!("{}: {day}", started.get_untracked()))
                    }
                    machine::Effect::Cancelled => selection_status.set(cancelled.get_untracked()),
                },
            );
            effect(move || {
                value.get();
                pick.send(machine::Event::ExternalValueChanged)
            });
            Choice::Range { value, pick }
        }
    };
    let count = policy.months;
    let disabled = policy.disabled.clone();
    let initial = policy.initial_focus;
    let first_day = policy.week_start.unwrap_or_else(|| {
        let locale = kivact_core::i18n::use_locale();
        memo(move || locale.get().first_day_of_week())
    });
    let days = memo(move || {
        let first = month.get();
        (0..count as i32)
            .map(|m| first.add_months(m))
            .flat_map(|month| {
                (0..i32::from(month.days())).map(move |d| month.first_day().add_days(d))
            })
            .collect::<Vec<_>>()
    });
    let active = memo(move || {
        let days = days.try_get()?;
        registry.track();
        let available = |day: &CalendarDate| {
            days.contains(day)
                && !disabled(*day)
                && registry
                    .with_members(|members| members.get(day).is_some_and(|entry| entry.available()))
        };
        let anchor = match choice {
            Choice::Single(value) => value.get(),
            Choice::Range { value, .. } => value.get().map(DateRange::start),
        };
        [requested.get(), anchor, initial]
            .into_iter()
            .flatten()
            .find(|day| available(day))
            .or_else(|| days.iter().copied().find(|day| available(day)))
    });
    let state = Rc::new(State {
        chosen_message: chosen,
        selection_status,
        grid_state: signal(None),
        cell_states: signal(std::collections::HashMap::new()),
        owner,
        root,
        month,
        choice,
        policy,
        first_day,
        roving,
        active,
    });
    let stored = owner.store("CalendarGridApi", state.clone());
    effect(move || {
        if let Some(state) = stored.try_get() {
            reconcile(&state);
        }
    });
    stored
}

/// Follow focus that reaches a day by other means, and give the focus of a
/// removed day to the tab stop.
fn reconcile(state: &State) {
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
            || match self.choice {
                Choice::Range { pick, .. } => match pick.get() {
                    RangePick::Picking { anchor, .. } => {
                        let span = anchor.to_days().abs_diff(day.to_days()) + 1;
                        day != anchor
                            && (span < u64::from(self.policy.min_span.get().max(1))
                                || self
                                    .policy
                                    .max_span
                                    .get()
                                    .is_some_and(|max| span > u64::from(max)))
                    }
                    RangePick::Idle => false,
                },
                Choice::Single(_) => false,
            }
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
        let done = match self.choice {
            Choice::Single(value) => {
                value.set(Some(day));
                self.selection_status
                    .set(format!("{}: {day}", self.chosen_message.get_untracked()));
                true
            }
            Choice::Range { pick, .. } => {
                if matches!(pick.get(),RangePick::Picking{anchor,..} if anchor==day)
                    && self.policy.min_span.get() > 1
                {
                    pick.send(machine::Event::Cancel);
                    return;
                }
                let done = matches!(pick.get(), RangePick::Picking { .. });
                pick.send(machine::Event::Choose { day });
                done
            }
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
        let modifiers = event.modifiers();
        if modifiers.ctrl || modifiers.alt || modifiers.meta {
            return Handled::No;
        }
        if event.key() == Key::Escape
            && let Choice::Range { value, pick } = self.choice
            && let RangePick::Picking { .. } = pick.get()
        {
            pick.send(machine::Event::Cancel);
            return Handled::Yes;
        }
        let column = day.weekday().days_since(self.first_day.get_untracked()) as i32;
        let months = if modifiers.shift { 12 } else { 1 };
        let (from, step) = match (event.key(), GridMove::from_key(event.key(), modifiers)) {
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
                && let Choice::Single(value) = self.choice
            {
                value.set(Some(to));
            }
        }
        Handled::Yes
    }
}
