//! Role: The Calendar builder and the month panels view it builds.

use super::state::CalendarCaption;
use super::state::Shared;
use kivact_core::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact_core::state::IntoBinding;
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Callback, Value, on_mount, signal};
use kivact_core::style::style;
use kivact_core::view::{IntoView, Role, View, dynamic};
use kivact_core::view::{Node, NodeBuilder, node, row};
use kivact_interaction::{CalendarOptions, CalendarSelection, use_calendar_grid};
use std::rc::Rc;

/// A month calendar (`Calendar`).
///
/// Created by [`calendar`] with a single-date or range binding. The days form a grid: the arrow keys move by a day or a week, Home
/// and End go to the start and end of the week, PageUp and PageDown move a
/// month (a year with Shift), and Enter, Space or a click chooses the day.
/// Dates the [`disabled_dates`](Calendar::disabled_dates) predicate rejects
/// can be neither focused nor chosen. The shown month is the calendar's own
/// (it starts at [`default_month`](Calendar::default_month), the month of
/// the selection or this month) and its arrows, captions and keys move it.
/// Month and weekday names come from the current locale; navigation labels
/// use the `calendar.previous`, `calendar.next`, `calendar.month` and
/// `calendar.year` messages.
// xtask-allow(state-fields): Setup builder options are independent bindings, callbacks and display capabilities.
pub struct Calendar {
    root: Node,
    selection: CalendarSelection,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    month: Option<CalendarMonth>,
    disabled: Rc<dyn Fn(CalendarDate) -> bool>,
    count: Value<usize>,
    caption: Value<CalendarCaption>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    outside: Value<bool>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    week_start: Option<Value<Weekday>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    on_commit: Option<Callback<()>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    autofocus: Value<bool>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    displayed_month: Option<Binding<CalendarMonth>>,
    today: Value<CalendarDate>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    min: Value<Option<CalendarDate>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    max: Value<Option<CalendarDate>>,
    min_span: Value<u32>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    max_span: Value<Option<u32>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    variable_rows: Value<bool>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    marks: Rc<dyn Fn(CalendarDate) -> Option<kivact_interaction::Tone>>,
    events: Value<Vec<super::state::CalendarEvent>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    weekend: Rc<dyn Fn(Weekday) -> Option<kivact_interaction::Tone>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    year_navigation: Value<bool>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    month_title: Option<Rc<dyn Fn(CalendarDate) -> String>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    range: Value<Option<DateRange>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    on_activate: Option<Callback<CalendarDate>>,
}

fn new(selection: CalendarSelection) -> Calendar {
    Calendar {
        root: crate::layer::with_base_recipe_style(
            node().with_node(|root| root.role(Role::Group)),
            |r| r.calendar_recipes().root_recipe(),
            &crate::variants::Variants::default(),
            kivact_core::custom::PartState::default,
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
        events: Vec::new().into_value(),
        weekend: Rc::new(|_| None),
        year_navigation: false.into_value(),
        month_title: None,
        range: None.into_value(),
        on_activate: None,
    }
}

/// A calendar with a single-day or ordered range selection.
/// Arrow keys navigate days; Page Up and Page Down navigate months.
#[track_caller]
pub fn calendar(selected: impl Into<CalendarSelection>) -> Calendar {
    new(selected.into())
}

impl Calendar {
    pub(crate) fn range_validator(&self) -> impl Fn(DateRange) -> bool + use<> {
        let (min, max, min_span, max_span) = (self.min, self.max, self.min_span, self.max_span);
        let disabled = self.disabled.clone();
        move |range| {
            let (start, end) = (range.start(), range.end());
            let span = start.to_days().abs_diff(end.to_days()) + 1;
            min.get().is_none_or(|min| start >= min)
                && max.get().is_none_or(|max| end <= max)
                && !disabled(start)
                && !disabled(end)
                && span >= u64::from(min_span.get().max(1))
                && max_span.get().is_none_or(|max| span <= u64::from(max))
        }
    }
    /// Up to three event dots for each date.
    pub fn events(mut self, events: impl IntoValue<Vec<super::state::CalendarEvent>>) -> Self {
        self.events = events.into_value();
        self
    }
    /// Semantic weekday tones, independent of date-specific marks.
    pub fn weekend(
        mut self,
        weekend: impl Fn(Weekday) -> Option<kivact_interaction::Tone> + 'static,
    ) -> Self {
        self.weekend = Rc::new(weekend);
        self
    }
    /// Show previous-year and next-year buttons.
    pub fn year_navigation(mut self, on: impl IntoValue<bool>) -> Self {
        self.year_navigation = on.into_value();
        self
    }
    /// Format the month heading from its first date.
    pub fn month_title(mut self, title: impl Fn(CalendarDate) -> String + 'static) -> Self {
        self.month_title = Some(Rc::new(title));
        self
    }
    /// Highlight an additional ordered range, without editing it.
    pub fn range(mut self, range: impl IntoValue<Option<DateRange>>) -> Self {
        self.range = range.into_value();
        self
    }
    /// Run after an explicit day activation, including an already selected date.
    pub fn on_activate<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(CalendarDate) -> R + 'static,
    ) -> Self {
        self.on_activate = Some(kivact_core::state::callback(f));
        self
    }

    /// Control the first displayed month; paging writes this binding.
    pub fn month(mut self, month: impl IntoBinding<CalendarMonth>) -> Self {
        self.displayed_month = Some(month.into_binding());
        self
    }
    /// The day drawn as today, read reactively.
    pub fn today(mut self, day: impl IntoValue<CalendarDate>) -> Self {
        self.today = day.into_value();
        self
    }
    /// Earliest selectable date, inclusive.
    pub fn min(mut self, day: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.min = day.into_value();
        self
    }
    /// Latest selectable date, inclusive.
    pub fn max(mut self, day: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.max = day.into_value();
        self
    }
    /// Set both inclusive bounds.
    pub fn bounds(
        self,
        min: impl IntoValue<Option<CalendarDate>>,
        max: impl IntoValue<Option<CalendarDate>>,
    ) -> Self {
        self.min(min).max(max)
    }
    /// Limit a completed range's inclusive day count.
    pub fn range_span(
        mut self,
        min: impl IntoValue<u32>,
        max: impl IntoValue<Option<u32>>,
    ) -> Self {
        self.min_span = min.into_value();
        self.max_span = max.into_value();
        self
    }
    /// Render only the weeks occupied by this month when true.
    pub fn variable_rows(mut self, value: impl IntoValue<bool>) -> Self {
        self.variable_rows = value.into_value();
        self
    }
    /// Add a semantic mark to a date without affecting selection.
    pub fn marks(
        mut self,
        marks: impl Fn(CalendarDate) -> Option<kivact_interaction::Tone> + 'static,
    ) -> Self {
        self.marks = Rc::new(marks);
        self
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
    /// click of a range selection.
    pub fn on_commit<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn() -> R + 'static,
    ) -> Self {
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
            autofocus,
            displayed_month,
            today,
            min,
            max,
            min_span,
            max_span,
            variable_rows,
            marks,
            events,
            weekend,
            year_navigation,
            month_title,
            range,
            on_activate,
        } = self;

        let anchor = match selection {
            CalendarSelection::Single(value) => value.get_untracked(),
            CalendarSelection::Range(value) => value.get_untracked().map(|r| r.start()),
        };
        let focus_day = anchor.unwrap_or_else(|| today.get_untracked());
        let month: Binding<CalendarMonth> = displayed_month
            .unwrap_or_else(|| signal(month.unwrap_or(focus_day.month_of())).into_binding());
        let locale = kivact_core::i18n::use_locale();
        let base_month = month;
        let navigation = kivact_core::state::machine(
            String::new(),
            move || super::navigation::MonthSnapshot {
                locale: locale.get(),
            },
            move |_, next| base_month.set(next),
        );
        kivact_core::state::effect(move || {
            navigation.send(super::navigation::MonthEvent::Observe(base_month.get()))
        });
        let month = kivact_core::state::binding(base_month, move |month| {
            navigation.send(super::navigation::MonthEvent::Navigate(month))
        });
        let rtl = kivact_core::view::use_direction();
        let swipe = kivact_core::state::machine(
            super::navigation::Swipe::Idle,
            move || super::navigation::SwipeSnapshot {
                rtl: rtl.get() == kivact_core::view::ReadingDirection::Rtl,
            },
            move |_, shift| {
                let next = base_month.get_untracked().add_months(shift);
                let allowed = min.get().is_none_or(|min| next >= min.month_of())
                    && max.get().is_none_or(|max| next <= max.month_of());
                if allowed {
                    month.set(next);
                }
            },
        );
        let autofocus = kivact_core::state::machine(
            super::navigation::InitialFocus::Pending,
            move || super::navigation::FocusSnapshot {
                requested: autofocus.get(),
            },
            |_, grid| {
                grid.focus_active();
            },
        );
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
                        move |day| {
                            disabled(day)
                                || min.get().is_some_and(|min| day < min)
                                || max.get().is_some_and(|max| day > max)
                        }
                    })
                    .range_span(min_span, max_span)
                    .months(count)
                    .initial_focus(focus_day);
                if let Some(day) = week_start {
                    policy = policy.week_start(day);
                }
                policy = policy.on_activate(move |day| {
                    if let Some(commit) = on_commit {
                        commit.call(());
                    }
                    if let Some(activate) = on_activate {
                        activate.call(day);
                    }
                });
                let grid = use_calendar_grid(month, selection, policy);
                let first = grid.week_start();
                let shared = Rc::new(Shared {
                    grid,
                    month,
                    count,
                    caption,
                    outside,
                    today,
                    variable_rows,
                    marks: marks.clone(),
                    events,
                    weekend: weekend.clone(),
                    year_navigation,
                    month_title: month_title.clone(),
                    range,
                });
                let names = Rc::new(super::caption::Names::new());
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
                        on_mount(move || autofocus.send(grid));
                        row(panels).style(style().gap(16))
                    },
                )
            },
        );
        let announcement = navigation.state();
        let status = kivact_core::view::text(kivact_core::i18n::message_with(
            "calendar.showing_month",
            "Showing {month}",
            move || [("month", announcement.get())],
        ))
        .with_node(|n| n.role(Role::Status))
        .live_region(kivact_core::view::LiveRegion::Polite)
        .style(
            style()
                .position(kivact_core::style::Position::Absolute)
                .size(1, 1)
                .opacity(0.0),
        );
        root.on_pointer(move |event| {
            swipe.send(super::navigation::Event::Pointer(event));
            if matches!(swipe.get(), super::navigation::Swipe::Dragging { .. }) {
                event.capture();
                kivact_core::view::Handled::Yes
            } else {
                kivact_core::view::Handled::No
            }
        })
        .child((body, status))
        .into_view()
    }
}
