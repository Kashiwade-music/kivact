//! Role: The Calendar builder and the month panels view it builds.

use super::state::CalendarCaption;
use super::state::Shared;
use crate::ColorName;
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact::interaction::{CalendarOptions, CalendarSelection, use_calendar_grid};
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::state::{Binding, Callback, Value, on_mount, signal};
use kivact::style::{Direction, style};
use kivact::view::{IntoView, Role, View, dynamic};
use kivact::view::{Node, NodeBuilder, node, row};
use std::rc::Rc;

/// A month calendar (`Calendar`).
///
/// Created by [`calendar`] (one day) or [`calendar_range`] (two clicks make a
/// range). The days form a grid: the arrow keys move by a day or a week, Home
/// and End go to the start and end of the week, PageUp and PageDown move a
/// month (a year with Shift), and Enter, Space or a click chooses the day.
/// Dates the [`disabled_dates`](Calendar::disabled_dates) predicate rejects
/// can be neither focused nor chosen. The shown month is the calendar's own
/// (it starts at [`default_month`](Calendar::default_month), the month of
/// the selection or this month) and its arrows, captions and keys move it.
/// The weekday and navigation texts are
/// messages: `calendar.sun` to `calendar.sat`, `calendar.previous` and
/// `calendar.next`, `calendar.month` and `calendar.year`, and the month names
/// `calendar.month_1` to `calendar.month_12` with their short forms
/// `calendar.short_1` to `calendar.short_12`.
pub struct Calendar {
    root: Node,
    selection: CalendarSelection,
    month: Option<CalendarMonth>,
    disabled: Rc<dyn Fn(CalendarDate) -> bool>,
    count: Value<usize>,
    caption: Value<CalendarCaption>,
    outside: Value<bool>,
    week_start: Option<Value<Weekday>>,
    on_commit: Option<Callback<()>>,
    autofocus: bool,
}

fn new(selection: CalendarSelection) -> Calendar {
    Calendar {
        root: node().with_node(|root| root.role(Role::Group)).style(
            style()
                .direction(Direction::Row)
                .gap(16)
                .padding(8)
                .shrink(0)
                .background(ColorName::Background)
                .color(ColorName::Foreground)
                .font_size(12)
                .align_self(kivact::style::Align::Start),
        ),
        selection,
        month: None,
        disabled: Rc::new(|_| false),
        count: 1usize.into_value(),
        caption: CalendarCaption::Label.into_value(),
        outside: true.into_value(),
        week_start: None,
        on_commit: None,
        autofocus: false,
    }
}

/// A calendar that chooses one day.
///
/// ```no_run
/// use kivact::i18n::CalendarDate;
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn pick() -> impl IntoView {
///     let day = signal(Some(CalendarDate::new(2026, 10, 3)));
///     basic::calendar(day)
/// }
/// ```
#[track_caller]
pub fn calendar(selected: impl kivact::state::IntoBinding<Option<CalendarDate>>) -> Calendar {
    new(CalendarSelection::Single(selected.into_binding()))
}

/// A calendar that chooses a range of days with two clicks.
///
/// After the first click the range is that one day and the next click
/// completes it, in either order.
#[track_caller]
pub fn calendar_range(range: impl kivact::state::IntoBinding<Option<DateRange>>) -> Calendar {
    new(CalendarSelection::Range(range.into_binding()))
}

impl Calendar {
    /// The month shown first (default: the month of the selection, or this
    /// month); the calendar moves the shown month from there.
    pub fn default_month(mut self, month: CalendarMonth) -> Self {
        self.month = Some(month);
        self
    }

    /// Dates for which `f` is true cannot be focused or chosen.
    pub fn disabled_dates(mut self, f: impl Fn(CalendarDate) -> bool + 'static) -> Self {
        self.disabled = Rc::new(f);
        self
    }

    /// How many months show side by side (default 1).
    pub fn number_of_months(mut self, count: impl IntoValue<usize>) -> Self {
        self.count = count.into_value();
        self
    }

    /// The month title: a label (default) or month and year selects.
    pub fn caption(mut self, caption: impl IntoValue<CalendarCaption>) -> Self {
        self.caption = caption.into_value();
        self
    }

    /// Show the days of the neighbouring months in the first and last week
    /// (default true; constant or reactive).
    #[track_caller]
    pub fn show_outside_days(mut self, show: impl IntoValue<bool>) -> Self {
        self.outside = show.into_value();
        self
    }

    /// The first column of the week (default: the locale's first day).
    pub fn week_start(mut self, day: impl IntoValue<Weekday>) -> Self {
        self.week_start = Some(day.into_value());
        self
    }

    /// Move the focus to the day that owns the tab stop once mounted.
    pub(crate) fn autofocus(mut self, on: bool) -> Self {
        self.autofocus = on;
        self
    }

    /// Call `f` when a choice is complete: a day for [`calendar`], the second
    /// click of [`calendar_range`].
    pub(crate) fn on_commit(mut self, f: impl Into<Callback<()>>) -> Self {
        self.on_commit = Some(f.into());
        self
    }
}

impl NodeBuilder for Calendar {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Calendar {
    fn into_view(self) -> View {
        let Calendar {
            root,
            selection,
            month,
            disabled,
            count,
            caption,
            outside,
            week_start,
            on_commit,
            autofocus,
        } = self;
        let today = CalendarDate::today();
        let anchor = match selection {
            CalendarSelection::Single(value) => value.get_untracked(),
            CalendarSelection::Range(value) => value.get_untracked().map(|r| r.start()),
        };
        let focus_day = anchor.unwrap_or(today);
        let month: Binding<CalendarMonth> =
            signal(month.unwrap_or(focus_day.month_of())).into_binding();
        let autofocus = Rc::new(std::cell::Cell::new(autofocus));
        let body = dynamic(
            move || {
                (
                    count.get().clamp(1, 12),
                    caption.get(),
                    week_start.map(|day| day.get()),
                )
            },
            move |(count, caption, week_start)| {
                let mut policy = CalendarOptions::new()
                    .disabled({
                        let disabled = disabled.clone();
                        move |day| disabled(day)
                    })
                    .months(count)
                    .initial_focus(focus_day);
                if let Some(day) = week_start {
                    policy = policy.week_start(day);
                }
                if let Some(commit) = on_commit {
                    policy = policy.on_activate(move |_| commit.call(()));
                }
                let grid = use_calendar_grid(month, selection, policy);
                let first = grid.week_start();
                let shared = Rc::new(Shared {
                    grid,
                    month,
                    count,
                    caption,
                    outside,
                    today,
                });
                let names = Rc::new(super::caption::Names::new());
                let autofocus = autofocus.clone();
                dynamic(
                    move || (month.get(), first.get()),
                    move |(month, first)| {
                        let panels = (0..shared.count)
                            .map(|i| {
                                super::panel::month_panel(
                                    &shared,
                                    &names,
                                    i,
                                    month.add_months(i as i32),
                                    first,
                                )
                            })
                            .collect::<Vec<_>>();
                        if autofocus.replace(false) {
                            on_mount(move || {
                                grid.focus_active();
                            });
                        }
                        row(panels).style(style().gap(16))
                    },
                )
            },
        );
        root.child(body).into_view()
    }
}
