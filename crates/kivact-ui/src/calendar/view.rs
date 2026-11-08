//! Role: The Calendar builder and the month panels view it builds.

use super::state::Shared;
use crate::calendar_values::CalendarCaption;
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
    cell_size: Value<f64>,
    variants: crate::variants::Variants,
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
    events: Value<Vec<crate::calendar_values::CalendarEvent>>,
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
        root: node().with_node(|root| root.role(Role::Group)),
        variants: crate::variants::Variants::default(),
        cell_size: kivact_core::state::memo(|| {
            kivact_core::style::use_theme()
                .get()
                .ext::<crate::UiTokens>()
                .calendar
                .cell_size
        }),
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
    pub(crate) fn sheet_month(&self, fallback: CalendarMonth) -> Binding<CalendarMonth> {
        self.displayed_month.unwrap_or_else(|| {
            let mut month = self.month.unwrap_or(fallback);
            if let Some(min) = self.min.get_untracked() {
                month = month.max(min.month_of());
            }
            if let Some(max) = self.max.get_untracked() {
                month = month.min(max.month_of());
            }
            signal(month).into_binding()
        })
    }
    pub(crate) fn touch(mut self) -> Self {
        let size = self.cell_size;
        self.cell_size = kivact_core::state::memo(move || size.get().max(44.0));
        self
    }

    /// Pass a kit-defined appearance to every calendar part recipe.
    pub fn variant(mut self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }

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
    pub fn events(
        mut self,
        events: impl IntoValue<Vec<crate::calendar_values::CalendarEvent>>,
    ) -> Self {
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
        let anchor = match self.selection {
            CalendarSelection::Single(value) => value.get_untracked(),
            CalendarSelection::Range(value) => value.get_untracked().map(|r| r.start()),
        };
        let focus_day = anchor.unwrap_or_else(|| self.today.get_untracked());
        let month: Binding<CalendarMonth> = self
            .displayed_month
            .unwrap_or_else(|| signal(self.month.unwrap_or(focus_day.month_of())).into_binding());
        let base_month = month;
        let (month, announcement) = month_navigation(base_month);
        let swipe = swipe_navigation(base_month, month, self.min, self.max);
        let autofocus = initial_focus(self.autofocus);
        let rules = CalendarRules {
            disabled: self.disabled.clone(),
            min: self.min,
            max: self.max,
            min_span: self.min_span,
            max_span: self.max_span,
            on_commit: self.on_commit,
            on_activate: self.on_activate,
        };
        let body = calendar_body(&self, month, rules, focus_day, autofocus, swipe);
        let status = kivact_core::view::node()
            .name(kivact_core::i18n::message_with(
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
        crate::layer::with_base_recipe_style(
            self.root,
            |r| r.calendar_recipes().root_recipe(),
            &self.variants,
            kivact_core::custom::PartState::default,
        )
        .on_pointer(move |event| {
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

fn range_prompt(
    grid: kivact_interaction::CalendarGridApi,
    selection: CalendarSelection,
) -> Option<Node> {
    if !matches!(selection, CalendarSelection::Range(_)) {
        return None;
    }
    let picking = grid.is_picking_range();
    let start = kivact_core::i18n::message("calendar.range.pick_start", "Pick a start date");
    let end = kivact_core::i18n::message("calendar.range.pick_end", "Pick an end date");
    Some(
        kivact_core::view::text(move || {
            if picking.get() {
                end.get()
            } else {
                start.get()
            }
        })
        .role(Role::Status)
        .live_region(kivact_core::view::LiveRegion::Polite)
        .style(
            style()
                .font_size(12)
                .color(kivact_core::style::ColorToken::TextMuted),
        ),
    )
}

// xtask-allow(state-fields): Independent construction constraints and commit observers; CalendarGridApi owns selection and range picking.
struct CalendarRules {
    disabled: Rc<dyn Fn(CalendarDate) -> bool>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    min_span: Value<u32>,
    max_span: Value<Option<u32>>,
    on_commit: Option<Callback<()>>,
    on_activate: Option<Callback<CalendarDate>>,
}
/// Connect month navigation announcements to its authoritative displayed binding.
fn month_navigation(base_month: Binding<CalendarMonth>) -> (Binding<CalendarMonth>, Value<String>) {
    let locale = kivact_core::i18n::use_locale();
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
    (month, navigation.state())
}
/// Apply horizontal swipe commands within the caller's date bounds.
fn swipe_navigation(
    base_month: Binding<CalendarMonth>,
    month: Binding<CalendarMonth>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
) -> kivact_core::state::MachineHandle<super::navigation::SwipeSnapshot> {
    let rtl = kivact_core::view::use_direction();
    kivact_core::state::machine(
        super::navigation::Swipe::Idle,
        move || {
            let current = base_month.get();
            super::navigation::SwipeSnapshot {
                rtl: rtl.get() == kivact_core::view::ReadingDirection::Rtl,
                now: kivact_core::custom::clock_now(),
                at_min: min.get().is_some_and(|min| current <= min.month_of()),
                at_max: max.get().is_some_and(|max| current >= max.month_of()),
            }
        },
        move |_, months| {
            // A flick turns as many months as it projects, stopping at the
            // bounds; it never moves the other way.
            let current = base_month.get_untracked();
            let next = current.add_months(months);
            let (next, moves) = if months > 0 {
                let next = max
                    .get_untracked()
                    .map_or(next, |max| next.min(max.month_of()));
                (next, next > current)
            } else {
                let next = min
                    .get_untracked()
                    .map_or(next, |min| next.max(min.month_of()));
                (next, next < current)
            };
            if moves {
                month.set(next);
            }
        },
    )
}
/// Scope the one-shot initial calendar focus request to the mounted owner.
fn initial_focus(
    requested: Value<bool>,
) -> kivact_core::state::MachineHandle<super::navigation::FocusSnapshot> {
    kivact_core::state::machine(
        super::navigation::InitialFocus::Pending,
        move || super::navigation::FocusSnapshot {
            requested: requested.get(),
        },
        |_, grid| {
            grid.focus_active();
        },
    )
}
/// Build grid policy from independent disabled-date, range and activation inputs.
fn calendar_policy(
    rules: &CalendarRules,
    count: usize,
    week_start: Option<Weekday>,
    focus_day: CalendarDate,
) -> CalendarOptions {
    let (min, max) = (rules.min, rules.max);
    let mut policy = CalendarOptions::new()
        .disabled({
            let disabled = rules.disabled.clone();
            move |day| {
                disabled(day)
                    || min.get().is_some_and(|min| day < min)
                    || max.get().is_some_and(|max| day > max)
            }
        })
        .range_span(rules.min_span, rules.max_span)
        .months(count)
        .initial_focus(focus_day);
    if let Some(day) = week_start {
        policy = policy.week_start(day);
    }
    let on_commit = rules.on_commit;
    let on_activate = rules.on_activate;
    policy = policy.on_activate(move |day| {
        if let Some(commit) = on_commit {
            commit.call(());
        }
        if let Some(activate) = on_activate {
            activate.call(day);
        }
    });
    policy
}
/// Render the shared month panels and forward the initial focus event after mount.
fn month_panels(
    shared: Rc<Shared>,
    autofocus: kivact_core::state::MachineHandle<super::navigation::FocusSnapshot>,
) -> View {
    let grid = shared.grid;
    let month = shared.month;
    let first = grid.week_start();
    let names = Rc::new(super::caption::Names::new());
    let swipe = shared.swipe;
    let rtl = kivact_core::view::use_direction();
    dynamic(
        move || (month.get(), first.get()),
        move |(month, first)| {
            let entering = swipe
                .state()
                .get_untracked()
                .entering(rtl.get_untracked() == kivact_core::view::ReadingDirection::Rtl);
            let panels = (0..shared.count)
                .map(|i| {
                    let shown = month.add_months(i as i32);
                    super::panel::month_panel(&shared, &names, i, shown, first, entering)
                })
                .collect::<Vec<_>>();
            on_mount(move || {
                autofocus.send(grid);
                swipe.send(super::navigation::Event::Entered);
            });
            row(panels).style(style().gap(16))
        },
    )
    .into_view()
}

/// Construct the reactive month body from independent calendar presentation inputs.
fn calendar_body(
    calendar: &Calendar,
    month: Binding<CalendarMonth>,
    rules: CalendarRules,
    focus_day: CalendarDate,
    autofocus: kivact_core::state::MachineHandle<super::navigation::FocusSnapshot>,
    swipe: kivact_core::state::MachineHandle<super::navigation::SwipeSnapshot>,
) -> View {
    let count = calendar.count;
    let cell_size = calendar.cell_size;
    let caption = calendar.caption;
    let week_start = calendar.week_start;
    let selection = calendar.selection;
    let outside = calendar.outside;
    let today = calendar.today;
    let variable_rows = calendar.variable_rows;
    let events = calendar.events;
    let year_navigation = calendar.year_navigation;
    let range = calendar.range;
    let variants = calendar.variants.clone();
    let marks = calendar.marks.clone();
    let weekend = calendar.weekend.clone();
    let month_title = calendar.month_title.clone();
    dynamic(
        move || {
            (
                count.get().clamp(1, 12),
                caption.get(),
                week_start.map(|day| day.get()),
                cell_size.get(),
            )
        },
        move |(count, caption, week_start, cell_size)| {
            let policy =
                calendar_policy(&rules, count, week_start, focus_day).activation_guard(move |_| {
                    !matches!(swipe.get(), super::navigation::Swipe::Released { .. })
                });
            let grid = use_calendar_grid(month, selection, policy);
            let shared = Rc::new(Shared {
                swipe,
                cell_size,
                variants: variants.clone(),
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
            let panels = month_panels(shared, autofocus);
            kivact_core::view::column((
                range_prompt(grid, selection),
                panels,
                kivact_core::view::node()
                    .role(Role::Status)
                    .name(grid.selection_status())
                    .live_region(kivact_core::view::LiveRegion::Polite),
            ))
        },
    )
    .into_view()
}
