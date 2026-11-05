//! Role: The Calendar builder and the month panels view it builds.

use super::state::CalendarCaption;
use super::state::Shared;
use kivact_core::style::ColorToken;
use kivact_core::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact_interaction::{CalendarOptions, CalendarSelection, use_calendar_grid};
use kivact_core::state::IntoBinding;
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Callback, Value, on_mount, signal};
use kivact_core::style::{Direction, style};
use kivact_core::view::{IntoView, Role, View, dynamic};
use kivact_core::view::{Node, NodeBuilder, node, row};
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
    autofocus: Value<bool>,
    displayed_month: Option<Binding<CalendarMonth>>,
    today: Value<CalendarDate>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    min_span: Value<u32>,
    max_span: Value<Option<u32>>,
    variable_rows: Value<bool>,
    marks: Rc<dyn Fn(CalendarDate) -> Option<kivact_interaction::Tone>>,
}

fn new(selection: CalendarSelection) -> Calendar {
    Calendar {
        root: node().with_node(|root| root.role(Role::Group)).style(
            style()
                .direction(Direction::Row)
                .gap(16)
                .padding(8)
                .shrink(0)
                .background(ColorToken::Surface)
                .color(ColorToken::Text)
                .font_size(12)
                .align_self(kivact_core::style::Align::Start),
        ),
        selection,
        month: None,
        disabled: Rc::new(|_| false),
        count: 1usize.into_value(),
        caption: CalendarCaption::Label.into_value(),
        outside: true.into_value(),
        week_start: None,
        on_commit: None,
        autofocus: false.into_value(),
        displayed_month: None,
        today: (move || CalendarDate::today()).into_value(),
        min: None.into_value(),
        max: None.into_value(),
        min_span: 1.into_value(),
        max_span: None.into_value(),
        variable_rows: false.into_value(),
        marks: Rc::new(|_| None),
    }
}

/// A calendar with a single-day or ordered range selection.
/// Arrow keys navigate days; Page Up and Page Down navigate months.
#[track_caller]
pub fn calendar(selected: impl Into<CalendarSelection>) -> Calendar {
    new(selected.into())
}

impl Calendar {
    /// Control the first displayed month; paging writes this binding.
    pub fn month(mut self, month: impl IntoBinding<CalendarMonth>) -> Self {
        self.displayed_month = Some(month.into_binding()); self
    }
    /// The day drawn as today, read reactively.
    pub fn today(mut self, day: impl IntoValue<CalendarDate>) -> Self {
        self.today = day.into_value(); self
    }
    /// Earliest selectable date, inclusive.
    pub fn min(mut self, day: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.min = day.into_value(); self
    }
    /// Latest selectable date, inclusive.
    pub fn max(mut self, day: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.max = day.into_value(); self
    }
    /// Set both inclusive bounds.
    pub fn bounds(self, min: impl IntoValue<Option<CalendarDate>>, max: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.min(min).max(max)
    }
    /// Limit a completed range's inclusive day count.
    pub fn range_span(mut self, min: impl IntoValue<u32>, max: impl IntoValue<Option<u32>>) -> Self {
        self.min_span = min.into_value(); self.max_span = max.into_value(); self
    }
    /// Render only the weeks occupied by this month when true.
    pub fn variable_rows(mut self, value: impl IntoValue<bool>) -> Self {
        self.variable_rows = value.into_value(); self
    }
    /// Add a semantic mark to a date without affecting selection.
    pub fn marks(mut self, marks: impl Fn(CalendarDate) -> Option<kivact_interaction::Tone> + 'static) -> Self {
        self.marks = Rc::new(marks); self
    }
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
    pub fn autofocus(mut self, on: impl IntoValue<bool>) -> Self {
        self.autofocus = on.into_value();
        self
    }

    /// Call `f` when a choice is complete: a day for [`calendar`], the second
    /// click of [`calendar_range`].
    pub fn on_commit<R: kivact_core::view::HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_commit = Some(kivact_core::state::callback(move |()| f()));
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
            autofocus, displayed_month, today, min, max, min_span, max_span, variable_rows, marks,
        } = self;

        let anchor = match selection {
            CalendarSelection::Single(value) => value.get_untracked(),
            CalendarSelection::Range(value) => value.get_untracked().map(|r| r.start()),
        };
        let focus_day = anchor.unwrap_or_else(|| today.get_untracked());
        let month: Binding<CalendarMonth> =
            displayed_month.unwrap_or_else(|| signal(month.unwrap_or(focus_day.month_of())).into_binding());
        let selection = match selection {
            CalendarSelection::Single(value) => CalendarSelection::Single(value),
            CalendarSelection::Range(value) => CalendarSelection::Range(kivact_core::state::binding(value, move |next: Option<DateRange>| {
                if next.is_none_or(|range| {
                    let span = range.start().to_days().abs_diff(range.end().to_days()) + 1;
                    span >= u64::from(min_span.get().max(1)) && max_span.get().is_none_or(|max| span <= u64::from(max))
                }) { value.set(next); }
            })),
        };
        let autofocus = Rc::new(std::cell::Cell::new(autofocus.get_untracked()));
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
                        move |day| disabled(day) || min.get().is_some_and(|min| day < min) || max.get().is_some_and(|max| day > max)
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
                    today, variable_rows, marks: marks.clone(),
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
