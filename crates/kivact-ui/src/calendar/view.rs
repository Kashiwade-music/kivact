//! Role: The Calendar component and the month panels view it builds.

use super::state::Shared;
use crate::calendar::CalendarCaption;
use crate::calendar::CalendarEvent;
use crate::calendar::look::CalendarTheme;
use kivact_core::custom::Caller;
use kivact_core::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday};
use kivact_core::state::{Callback, IntoValue, Signal, Value, memo, on_mount, signal};
use kivact_core::style::style;
use kivact_core::view::{Element, IntoView, LiveRegion, Node, Role, View, dynamic, node, row};
use kivact_interaction::{
    CalendarOptions, CalendarSelection, MonthSwipeApi, MonthSwipePhase, Tone, use_calendar_grid,
    use_month_swipe,
};
use kivact_macros::component;
use std::rc::Rc;

/// A month calendar (`Calendar`) with a single-day or ordered range selection.
///
/// Created by [`calendar`] with a single-date or range binding. The days form
/// a grid: the arrow keys move by a day or a week, Home and End go to the start
/// and end of the week, PageUp and PageDown move a month (a year with Shift),
/// and Enter, Space or a click chooses the day. Dates the
/// [`disabled_dates`](Calendar::disabled_dates) predicate rejects can be neither
/// focused nor chosen. The shown month is the calendar's own (it starts at
/// [`default_month`](Calendar::default_month), the month of the selection or
/// this month) and its arrows, captions and keys move it. Month and weekday
/// names come from the current locale; navigation labels use the
/// `calendar.previous`, `calendar.next`, `calendar.month` and `calendar.year`
/// messages.
///
/// Setters: `touch` (day cells at least 44 px), `default_month`,
/// `disabled_dates`, `number_of_months`, `caption`, `show_outside_days`,
/// `week_start`, `on_commit`, `autofocus`, `month` (the shown month, written
/// by paging), `today`, `min` and `max` (inclusive date bounds), `min_span`
/// and `max_span` (inclusive day count of a range), `variable_rows`, `marks`,
/// `events`, `weekend`, `year_navigation`, `month_title`, `range`, and
/// `on_activate`.
#[component]
pub fn calendar(
    selected: CalendarSelection,
    #[prop(default = false)] touch: bool,
    default_month: Option<CalendarMonth>,
    disabled_dates: Option<Rc<dyn Fn(CalendarDate) -> bool>>,
    #[prop(default = 1usize)] number_of_months: Value<usize>,
    #[prop(default = CalendarCaption::Label)] caption: Value<CalendarCaption>,
    #[prop(default = true)] show_outside_days: Value<bool>,
    week_start: Option<Value<Weekday>>,
    on_commit: Option<Callback<()>>,
    #[prop(default = false)] autofocus: Value<bool>,
    month: Option<Signal<CalendarMonth>>,
    today: Option<Value<CalendarDate>>,
    #[prop(default = None::<CalendarDate>)] min: Value<Option<CalendarDate>>,
    #[prop(default = None::<CalendarDate>)] max: Value<Option<CalendarDate>>,
    #[prop(default = 1u32)] min_span: Value<u32>,
    #[prop(default = None::<u32>)] max_span: Value<Option<u32>>,
    #[prop(default = false)] variable_rows: Value<bool>,
    marks: Option<Rc<dyn Fn(CalendarDate) -> Option<Tone>>>,
    events: Option<Value<Vec<CalendarEvent>>>,
    weekend: Option<Rc<dyn Fn(Weekday) -> Option<Tone>>>,
    #[prop(default = false)] year_navigation: Value<bool>,
    month_title: Option<Rc<dyn Fn(CalendarDate) -> String>>,
    #[prop(default = None::<DateRange>)] range: Value<Option<DateRange>>,
    on_activate: Option<Callback<CalendarDate>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let root = node().with_node(|root| root.role(Role::Group));
    let base = memo(|| {
        kivact_core::style::use_theme()
            .get()
            .get(&crate::calendar::look::CELL_SIZE_VAR)
    });
    let cell_size = if touch {
        memo(move || base.get().max(44.0))
    } else {
        base
    };
    let anchor = match selected {
        CalendarSelection::Single(value) => value.get_untracked(),
        CalendarSelection::Range(value) => value.get_untracked().map(|r| r.start()),
    };
    let today: Value<CalendarDate> =
        today.unwrap_or_else(|| (move || CalendarDate::today()).into_value());
    let focus_day = anchor.unwrap_or_else(|| today.get_untracked());
    let month: Signal<CalendarMonth> =
        month.unwrap_or_else(|| signal(default_month.unwrap_or(focus_day.month_of())));
    let announcement = month_announcement(month);
    let swipe = use_month_swipe(month, min, max);
    let autofocus = initial_focus(autofocus);
    let disabled = disabled_rule(disabled_dates);
    let marks: Rc<dyn Fn(CalendarDate) -> Option<Tone>> = match marks {
        Some(marks) => marks,
        None => Rc::new(|_| None),
    };
    let weekend: Rc<dyn Fn(Weekday) -> Option<Tone>> = match weekend {
        Some(weekend) => weekend,
        None => Rc::new(|_| None),
    };
    let events: Value<Vec<CalendarEvent>> =
        events.unwrap_or_else(|| Vec::<CalendarEvent>::new().into_value());
    let rules = CalendarRules {
        disabled,
        min,
        max,
        min_span,
        max_span,
        on_commit,
        on_activate,
    };
    let shown = Shown {
        count: number_of_months,
        caption,
        week_start,
        selection: selected,
        outside: show_outside_days,
        today,
        variable_rows,
        events,
        year_navigation,
        range,
        variants: variants.clone(),
        marks,
        weekend,
        month_title,
        cell_size,
    };
    let body = calendar_body(shown, month, rules, focus_day, autofocus, swipe);
    let status = kivact_core::view::node()
        .name(kivact_core::i18n::message_with(
            "calendar.showing_month",
            "Showing {month}",
            move || [("month", announcement.get())],
        ))
        .with_node(|n| n.role(Role::Status))
        .live_region(LiveRegion::Polite)
        .style(
            style()
                .position(kivact_core::style::Position::Absolute)
                .size(1, 1)
                .opacity(0.0),
        );
    crate::layer::with_base_recipe_style(
        root,
        |r: &CalendarTheme| r.root_recipe(),
        &variants,
        kivact_core::custom::PartState::default,
    )
    .part(swipe.area())
    .child((body, status))
}

/// The date rule a calendar applies to its days: none means every date is
/// enabled.
fn disabled_rule(
    rule: Option<Rc<dyn Fn(CalendarDate) -> bool>>,
) -> Rc<dyn Fn(CalendarDate) -> bool> {
    match rule {
        Some(rule) => rule,
        None => Rc::new(|_| false),
    }
}

/// The month a sheet shows first: the calendar's own shown month, or its
/// default month (or `fallback`), moved into its inclusive date bounds. The
/// sheet's wheels and title write the returned signal, which the calendar
/// shows.
pub(crate) fn sheet_month(calendar: &Calendar, fallback: CalendarMonth) -> Signal<CalendarMonth> {
    calendar.month.unwrap_or_else(|| {
        let mut month = calendar.default_month.unwrap_or(fallback);
        if let Some(min) = calendar.min.get_untracked() {
            month = month.max(min.month_of());
        }
        if let Some(max) = calendar.max.get_untracked() {
            month = month.min(max.month_of());
        }
        signal(month)
    })
}

/// The validator of a range the calendar offers: its inclusive bounds, its
/// day count and its disabled dates.
pub(crate) fn range_validator(calendar: &Calendar) -> impl Fn(DateRange) -> bool + use<> {
    let (min, max) = (calendar.min.clone(), calendar.max.clone());
    let (min_span, max_span) = (calendar.min_span.clone(), calendar.max_span.clone());
    let disabled = disabled_rule(calendar.disabled_dates.clone());
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
        .live_region(LiveRegion::Polite)
        .style(
            style()
                .font_size(12)
                .color(kivact_core::style::color::TEXT_MUTED),
        ),
    )
}

struct CalendarRules {
    disabled: Rc<dyn Fn(CalendarDate) -> bool>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    min_span: Value<u32>,
    max_span: Value<Option<u32>>,
    on_commit: Option<Callback<()>>,
    on_activate: Option<Callback<CalendarDate>>,
}

/// What the month body shows: the resolved presentation inputs of the calendar.
struct Shown {
    count: Value<usize>,
    caption: Value<CalendarCaption>,
    week_start: Option<Value<Weekday>>,
    selection: CalendarSelection,
    outside: Value<bool>,
    today: Value<CalendarDate>,
    variable_rows: Value<bool>,
    events: Value<Vec<CalendarEvent>>,
    year_navigation: Value<bool>,
    range: Value<Option<DateRange>>,
    variants: kivact_core::view::Variants,
    marks: Rc<dyn Fn(CalendarDate) -> Option<Tone>>,
    weekend: Rc<dyn Fn(Weekday) -> Option<Tone>>,
    month_title: Option<Rc<dyn Fn(CalendarDate) -> String>>,
    cell_size: Value<f64>,
}

/// The shown month's announcement: its long name and year in the locale.
fn month_announcement(month: Signal<CalendarMonth>) -> Value<String> {
    let locale = kivact_core::i18n::use_locale();
    memo(move || {
        let month = month.get();
        locale.with(|locale| {
            format!(
                "{} {}",
                locale.month_name(month.month(), kivact_core::i18n::NameWidth::Long),
                month.year()
            )
        })
    })
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
                .phase()
                .get_untracked()
                .entering(rtl == kivact_core::view::ReadingDirection::Rtl);
            let panels = (0..shared.count)
                .map(|i| {
                    let shown = month.add_months(i as i32);
                    super::panel::month_panel(&shared, &names, i, shown, first, entering)
                })
                .collect::<Vec<_>>();
            on_mount(move || {
                autofocus.send(grid);
                swipe.entered();
            });
            row(panels).style(style().gap(16))
        },
    )
    .into_view()
}

/// Construct the reactive month body from independent calendar presentation inputs.
fn calendar_body(
    shown: Shown,
    month: Signal<CalendarMonth>,
    rules: CalendarRules,
    focus_day: CalendarDate,
    autofocus: kivact_core::state::MachineHandle<super::navigation::FocusSnapshot>,
    swipe: MonthSwipeApi,
) -> View {
    let Shown {
        count,
        caption,
        week_start,
        selection,
        outside,
        today,
        variable_rows,
        events,
        year_navigation,
        range,
        variants,
        marks,
        weekend,
        month_title,
        cell_size,
    } = shown;
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
                    !matches!(swipe.phase().get(), MonthSwipePhase::Released { .. })
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
                    .live_region(LiveRegion::Polite),
            ))
        },
    )
    .into_view()
}
