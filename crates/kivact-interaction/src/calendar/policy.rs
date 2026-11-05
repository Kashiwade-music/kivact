//! Role: The selection and the policy a calendar grid is created with.

use kivact_core::i18n::{CalendarDate, DateRange, Weekday};
use kivact_core::state::{Binding, Callback, Value};
use kivact_core::state::{IntoValue, callback};
use std::rc::Rc;

/// The caller's authoritative selection: one day or a range.
#[derive(Clone, Copy)]
pub enum CalendarSelection {
    /// At most one chosen day.
    Single(Binding<Option<CalendarDate>>),
    /// At most one range of days.
    Range(Binding<Option<DateRange>>),
}

/// Disabled days, the number of months, the week and the commit rule of a
/// calendar grid.
///
/// Every method is a builder step; the default shows one month, starts the
/// week on the locale's first day of the week and disables nothing.
#[derive(Clone)]
pub struct CalendarOptions {
    pub(super) disabled: Rc<dyn Fn(CalendarDate) -> bool>,
    pub(super) activation_guard: Rc<dyn Fn(CalendarDate) -> bool>,
    pub(super) months: usize,
    pub(super) min_span: Value<u32>,
    pub(super) max_span: Value<Option<u32>>,
    pub(super) week_start: Option<Value<Weekday>>,
    pub(super) initial_focus: Option<CalendarDate>,
    pub(super) select_on_move: bool,
    pub(super) on_commit: Option<Callback<CalendarDate>>,
}

impl Default for CalendarOptions {
    fn default() -> Self {
        Self {
            disabled: Rc::new(|_| false),
            activation_guard: Rc::new(|_| true),
            months: 1,
            min_span: 1.into_value(),
            max_span: None.into_value(),
            week_start: None,
            initial_focus: None,
            select_on_move: false,
            on_commit: None,
        }
    }
}

impl CalendarOptions {
    /// The default policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Days for which `f` is true can be neither focused nor chosen. `f` may
    /// read signals; the days follow it.
    pub fn disabled(mut self, f: impl Fn(CalendarDate) -> bool + 'static) -> Self {
        self.disabled = Rc::new(f);
        self
    }

    /// Admit an activation before either selection transition runs, without
    /// changing the day's published availability. Gesture owners use this
    /// to suppress the trailing activation of a completed swipe.
    pub fn activation_guard(mut self, guard: impl Fn(CalendarDate) -> bool + 'static) -> Self {
        self.activation_guard = Rc::new(guard);
        self
    }

    /// How many months are shown side by side, 1 to 12 (default 1). Paging
    /// with the keys keeps the focused day among them.
    pub fn months(mut self, count: usize) -> Self {
        self.months = count.clamp(1, 12);
        self
    }

    /// Inclusive day-count bounds for completing a range. The first pick
    /// remains an anchor even when it is shorter than this minimum.
    pub fn range_span(
        mut self,
        min: impl IntoValue<u32>,
        max: impl IntoValue<Option<u32>>,
    ) -> Self {
        self.min_span = min.into_value();
        self.max_span = max.into_value();
        self
    }

    /// The first day of the week (default: the locale's, read reactively).
    pub fn week_start(mut self, day: impl IntoValue<Weekday>) -> Self {
        self.week_start = Some(day.into_value());
        self
    }

    /// The day that holds the tab stop when nothing is selected or moved to
    /// and it is among the shown months (default: the first enabled day).
    pub fn initial_focus(mut self, day: CalendarDate) -> Self {
        self.initial_focus = Some(day);
        self
    }

    /// With a single-day selection, a key that moves the focus also chooses
    /// the day it reaches, without running `on_commit` (default false: Enter
    /// or Space chooses). With a range it is a usage error.
    pub fn select_on_move(mut self, on: bool) -> Self {
        self.select_on_move = on;
        self
    }

    /// Run `f` with the day whose choice completes a selection: every choice
    /// of a single day (a day that is already chosen too), the second choice
    /// of a range. Arrow keys never commit.
    pub fn on_activate<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(CalendarDate) -> R + 'static,
    ) -> Self {
        self.on_commit = Some(callback(f));
        self
    }
}

impl From<kivact_core::state::Signal<Option<CalendarDate>>> for CalendarSelection {
    fn from(value: kivact_core::state::Signal<Option<CalendarDate>>) -> Self {
        use kivact_core::state::IntoBinding;
        Self::Single(value.into_binding())
    }
}
impl From<Binding<Option<CalendarDate>>> for CalendarSelection {
    fn from(value: Binding<Option<CalendarDate>>) -> Self {
        Self::Single(value)
    }
}
impl From<kivact_core::state::Signal<Option<DateRange>>> for CalendarSelection {
    fn from(value: kivact_core::state::Signal<Option<DateRange>>) -> Self {
        use kivact_core::state::IntoBinding;
        Self::Range(value.into_binding())
    }
}
impl From<Binding<Option<DateRange>>> for CalendarSelection {
    fn from(value: Binding<Option<DateRange>>) -> Self {
        Self::Range(value)
    }
}
