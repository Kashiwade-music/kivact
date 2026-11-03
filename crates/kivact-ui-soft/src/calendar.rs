//! Role: Soft calendar and its single-date and range selection surfaces.
use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, column, icon, node, row, text};
use kivact::i18n::{CalendarDate, CalendarMonth, DateRange, Weekday, message, message_with};
use kivact::interaction::{CalendarPolicy, CalendarSelection, CalendarSpan, use_calendar_grid};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Color, Justify, Length, Position, pct, style};
use kivact::ui::{ButtonVariant, button};
use kivact::view::{
    Handled, IntoValue, IntoView, LiveRegion, PointerEvent, PointerKind, PointerPhase, Role, View,
    dynamic,
};
use kivact::{Binding, binding, on_mount, signal};
use std::rc::Rc;

pub struct Calendar {
    root: Node,
    selection: CalendarSelection,
    month: Option<CalendarMonth>,
    disabled: Rc<dyn Fn(CalendarDate) -> bool>,
    count: usize,
    min: Option<CalendarDate>,
    max: Option<CalendarDate>,
    min_span: u32,
    max_span: Option<u32>,
    variable_rows: bool,
    outside: bool,
    week_start: Option<Weekday>,
    on_commit: Option<Rc<dyn Fn()>>,
    autofocus: bool,
}

fn new(selection: CalendarSelection) -> Calendar {
    Calendar {
        root: node().role(Role::Group).base_style(
            style()
                .padding(12)
                .width(Length::MaxContent)
                .max_width(pct(100)),
        ),
        selection,
        month: None,
        disabled: Rc::new(|_| false),
        count: 1,
        min: None,
        max: None,
        min_span: 1,
        max_span: None,
        variable_rows: false,
        outside: true,
        week_start: None,
        on_commit: None,
        autofocus: false,
    }
}

/// A calendar that selects one day.
pub fn calendar(value: impl Into<Binding<Option<CalendarDate>>>) -> Calendar {
    new(CalendarSelection::Single(value.into()))
}

/// A calendar that publishes a range after both ends have been chosen.
pub fn calendar_range(value: impl Into<Binding<Option<DateRange>>>) -> Calendar {
    new(CalendarSelection::Range(value.into()))
}

impl Calendar {
    pub fn default_month(mut self, month: CalendarMonth) -> Self {
        self.month = Some(month);
        self
    }

    pub fn disabled_dates(mut self, disabled: impl Fn(CalendarDate) -> bool + 'static) -> Self {
        self.disabled = Rc::new(disabled);
        self
    }

    /// Limit the selectable days to an inclusive interval.
    pub fn bounds(mut self, min: Option<CalendarDate>, max: Option<CalendarDate>) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Limit the inclusive span of a completed range, in days.
    pub fn range_span(mut self, min: u32, max: Option<u32>) -> Self {
        self.min_span = min.max(1);
        self.max_span = max.map(|max| max.max(self.min_span));
        self
    }

    pub(super) fn gate_disabled(mut self, off: kivact::Value<bool>) -> Self {
        let disabled = self.disabled;
        self.disabled = Rc::new(move |date| off.get() || disabled(date));
        self
    }

    pub(super) fn with_constraints(
        mut self,
        accept: impl Fn(CalendarDate) -> bool + 'static,
    ) -> Self {
        let disabled = self.disabled;
        self.disabled = Rc::new(move |date| !accept(date) || disabled(date));
        self
    }

    pub(super) fn range_validator(&self) -> impl Fn(DateRange) -> bool + use<> {
        let (min, max, min_span, max_span) = (self.min, self.max, self.min_span, self.max_span);
        let disabled = self.disabled.clone();
        move |range| {
            let (start, end) = (range.start(), range.end());
            let span = start.to_epoch_days().abs_diff(end.to_epoch_days()) + 1;
            min.is_none_or(|min| start >= min)
                && max.is_none_or(|max| end <= max)
                && span >= u64::from(min_span)
                && max_span.is_none_or(|max| span <= u64::from(max))
                && !disabled(start)
                && !disabled(end)
        }
    }

    /// Allow the number of displayed weeks to vary by month.
    pub fn variable_rows(mut self, variable: bool) -> Self {
        self.variable_rows = variable;
        self
    }

    pub fn number_of_months(mut self, count: usize) -> Self {
        self.count = count.clamp(1, 2);
        self
    }

    pub fn show_outside_days(mut self, show: bool) -> Self {
        self.outside = show;
        self
    }

    pub fn week_start(mut self, weekday: Weekday) -> Self {
        self.week_start = Some(weekday);
        self
    }

    pub fn autofocus(mut self, value: bool) -> Self {
        self.autofocus = value;
        self
    }

    pub fn on_commit(mut self, commit: impl Fn() + 'static) -> Self {
        self.on_commit = Some(Rc::new(commit));
        self
    }
}

impl NodeBuilder for Calendar {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

const MONTHS: [(&str, &str); 12] = [
    ("soft.calendar.month_1", "January"),
    ("soft.calendar.month_2", "February"),
    ("soft.calendar.month_3", "March"),
    ("soft.calendar.month_4", "April"),
    ("soft.calendar.month_5", "May"),
    ("soft.calendar.month_6", "June"),
    ("soft.calendar.month_7", "July"),
    ("soft.calendar.month_8", "August"),
    ("soft.calendar.month_9", "September"),
    ("soft.calendar.month_10", "October"),
    ("soft.calendar.month_11", "November"),
    ("soft.calendar.month_12", "December"),
];
const WEEKDAYS: [(&str, &str); 7] = [
    ("soft.calendar.mon", "Mo"),
    ("soft.calendar.tue", "Tu"),
    ("soft.calendar.wed", "We"),
    ("soft.calendar.thu", "Th"),
    ("soft.calendar.fri", "Fr"),
    ("soft.calendar.sat", "Sa"),
    ("soft.calendar.sun", "Su"),
];

pub(super) fn calendar_status(message: impl kivact::view::IntoText) -> Node {
    text(message)
        .role(Role::Status)
        .live_region(LiveRegion::Polite)
        .hit_testable(false)
        .style(style().position(Position::Absolute).size(1, 1).opacity(0.0))
}

fn memo_month_title(month: kivact::Value<CalendarMonth>) -> kivact::Value<String> {
    let names = MONTHS.map(|(key, default)| message(key, default));
    message_with("soft.calendar.caption", "{month} {year}", move || {
        let month = month.get();
        [
            ("month", names[usize::from(month.month()) - 1].get()),
            ("year", month.year().to_string()),
        ]
    })
}

fn title(month: CalendarMonth) -> kivact::Value<String> {
    let (key, default) = MONTHS[usize::from(month.month()) - 1];
    let name = message(key, default);
    message_with("soft.calendar.caption", "{month} {year}", move || {
        [("month", name.get()), ("year", month.year().to_string())]
    })
}

impl IntoView for Calendar {
    fn into_view(self) -> View {
        let Self {
            root,
            selection,
            month,
            disabled,
            count,
            min,
            max,
            min_span,
            max_span,
            variable_rows,
            outside,
            week_start,
            on_commit,
            autofocus,
        } = self;
        let today = CalendarDate::today();
        let focus = match selection {
            CalendarSelection::Single(value) => value.get_untracked(),
            CalendarSelection::Range(value) => value.get_untracked().map(|range| range.start()),
        }
        .unwrap_or(today);
        let focus_after_maximum = max.is_some_and(|max| focus > max);
        let focus = min.map_or(focus, |min| focus.max(min));
        let focus = max.map_or(focus, |max| focus.min(max));
        let month = month.unwrap_or_else(|| {
            let focused_month = focus.month_of();
            if focus_after_maximum && matches!(selection, CalendarSelection::Range(_)) {
                focused_month.add_months(1 - count as i32)
            } else {
                focused_month
            }
        });
        let month = min.map_or(month, |min| month.max(min.month_of()));
        let month = max.map_or(month, |max| month.min(max.month_of()));
        let month: Binding<CalendarMonth> = signal(month).into();
        let pending = signal(None::<CalendarDate>);
        let hovered = signal(None::<CalendarDate>);
        let announcement = signal(String::new());
        let chosen_message = message("soft.calendar.chosen", "Date selected");
        let started_message = message("soft.calendar.range_started", "Start date selected");
        let cancelled_message =
            message("soft.calendar.range_cancelled", "Range selection cancelled");
        let range_message = message("soft.calendar.range_chosen", "Date range selected");
        let month_value = month.into_value();
        let initial_month = month.get_untracked();
        let previous_month = std::cell::Cell::new(initial_month);
        let month_label = memo_month_title(month_value);
        kivact::effect(move || {
            let current = month_value.get();
            if previous_month.replace(current) != current {
                announcement.set(month_label.get_untracked());
            }
        });
        let mut policy = CalendarPolicy::new()
            .disabled(move |day| {
                min.is_some_and(|min| day < min)
                    || max.is_some_and(|max| day > max)
                    || disabled(day)
                    || pending.get().is_some_and(|anchor| {
                        let span = anchor.to_epoch_days().abs_diff(day.to_epoch_days()) + 1;
                        day != anchor
                            && (span < u64::from(min_span)
                                || max_span.is_some_and(|max| span > u64::from(max)))
                    })
            })
            .months(count)
            .initial_focus(focus);
        if let Some(weekday) = week_start {
            policy = policy.week_start(weekday);
        }
        let selection = match selection {
            CalendarSelection::Single(_) => {
                policy = policy.on_commit(move |day| {
                    announcement.set(format!("{}: {day}", chosen_message.get_untracked()));
                    if let Some(commit) = &on_commit {
                        commit();
                    }
                });
                selection
            }
            CalendarSelection::Range(value) => {
                let local = signal(value.get_untracked());
                kivact::effect(move || {
                    local.set(value.get());
                    pending.set(None);
                });
                let staged = binding(local, move |next: Option<DateRange>| {
                    local.set(next);
                    // The grid writes once for the anchor, then once more for
                    // either a completed choice or Escape's restored range.
                    let anchor = if pending.get_untracked().is_none() {
                        next.map(|range| range.start())
                    } else {
                        None
                    };
                    pending.set(anchor);
                    hovered.set(None);
                    if let Some(anchor) = anchor {
                        announcement.set(format!("{}: {anchor}", started_message.get_untracked()));
                    } else {
                        announcement.set(cancelled_message.get_untracked());
                    }
                });
                policy = policy.on_commit(move |_| {
                    let next = local.get_untracked();
                    if min_span > 1 && next.is_some_and(|range| range.start() == range.end()) {
                        local.set(value.get_untracked());
                        return;
                    }
                    value.set(next);
                    if let Some(range) = next {
                        announcement.set(format!(
                            "{}: {} – {}",
                            range_message.get_untracked(),
                            range.start(),
                            range.end()
                        ));
                    }
                    if let Some(commit) = &on_commit {
                        commit();
                    }
                });
                CalendarSelection::Range(staged)
            }
        };
        let grid = use_calendar_grid(month, selection, policy);
        let first = grid.week_start();
        let autofocus = std::cell::Cell::new(autofocus);
        let status = calendar_status(announcement);
        root.child(status)
            .child(dynamic(
                move || (month.get(), first.get()),
                move |(month, first)| {
                    let rtl =
                        kivact::view::current_direction() == kivact::view::ReadingDirection::Rtl;
                    let (back, forward) = if rtl {
                        ("chevron-right", "chevron-left")
                    } else {
                        ("chevron-left", "chevron-right")
                    };
                    let start = title(month);
                    let end = title(month.add_months(count as i32 - 1));
                    let previous = button(message("soft.calendar.previous", "Previous month"))
                        .variant(ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(icon(back))
                        .part(grid.previous_month())
                        .disabled(min.is_some_and(|min| month <= min.month_of()));
                    let next = button(message("soft.calendar.next", "Next month"))
                        .variant(ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(icon(forward))
                        .part(grid.next_month())
                        .disabled(max.is_some_and(|max| {
                            month.add_months(count as i32 - 1) >= max.month_of()
                        }));
                    let caption = row((
                        previous,
                        node()
                            .child(text(move || {
                                if count == 1 {
                                    start.get()
                                } else {
                                    format!("{} – {}", start.get(), end.get())
                                }
                            }))
                            .style(move || {
                                style()
                                    .grow(1)
                                    .align_items(Align::Center)
                                    .justify_content(Justify::Center)
                                    .font_size(14)
                                    .weight(600)
                                    .color(tokens().colors.text_primary)
                            }),
                        next,
                    ))
                    .style(style().height(32).align_items(Align::Center).gap(8));
                    let months = (0..count)
                        .map(|index| {
                            let month = month.add_months(index as i32);
                            let headers = row((0..7)
                                .map(|i| {
                                    let (key, label) =
                                        WEEKDAYS[first.days_after(i).days_since(Weekday::Monday)];
                                    node()
                                        .role(Role::ColumnHeader)
                                        .child(text(message(key, label)))
                                        .style(move || {
                                            style()
                                                .size(32, 36)
                                                .padding_bottom(4)
                                                .align_items(Align::Center)
                                                .justify_content(Justify::Center)
                                                .font_size(12)
                                                .color(tokens().colors.text_secondary)
                                        })
                                        .into_view()
                                })
                                .collect::<Vec<_>>())
                            .part(grid.row());
                            let weeks = month
                                .weeks(first, if variable_rows { 0 } else { 6 })
                                .into_iter()
                                .map(|week| {
                                    row(week
                                        .into_iter()
                                        .map(|date| {
                                            if date.month_of() != month {
                                                return node()
                                                    .child(text(if outside {
                                                        date.day().to_string()
                                                    } else {
                                                        String::new()
                                                    }))
                                                    .style(move || {
                                                        style()
                                                            .size(32, 32)
                                                            .align_items(Align::Center)
                                                            .justify_content(Justify::Center)
                                                            .font_size(14)
                                                            .color(tokens().colors.text_secondary)
                                                            .opacity(0.3)
                                                    })
                                                    .into_view();
                                            }
                                            let part = grid.cell(month, date);
                                            let state = part.state();
                                            let band = node().style(move || {
                                                let t = tokens();
                                                let band = style()
                                                    .position(Position::Absolute)
                                                    .top(2)
                                                    .height(28)
                                                    .background(t.colors.accent_muted);
                                                if let Some((anchor, end)) =
                                                    pending.get().zip(hovered.get())
                                                {
                                                    let (start, end) =
                                                        (anchor.min(end), anchor.max(end));
                                                    if start != end && (start..=end).contains(&date)
                                                    {
                                                        let lead = date == start
                                                            || date.weekday() == first
                                                            || date == month.first_day();
                                                        let tail = date == end
                                                            || date.weekday()
                                                                == first.days_after(6)
                                                            || date.day() == month.days();
                                                        let (left, right) = if rtl {
                                                            (tail, lead)
                                                        } else {
                                                            (lead, tail)
                                                        };
                                                        return band
                                                            .background(t.colors.overlay_hover)
                                                            .left(if left { 2 } else { 0 })
                                                            .width(
                                                                32 - if left { 2 } else { 0 }
                                                                    - if right { 2 } else { 0 },
                                                            )
                                                            .radius_corners(
                                                                if left { 14 } else { 0 },
                                                                if right { 14 } else { 0 },
                                                                if right { 14 } else { 0 },
                                                                if left { 14 } else { 0 },
                                                            );
                                                    }
                                                }
                                                match state.get().span {
                                                    CalendarSpan::Start if rtl => {
                                                        band.left(0).width(16)
                                                    }
                                                    CalendarSpan::End if rtl => {
                                                        band.left(16).width(16)
                                                    }
                                                    CalendarSpan::Start => band.left(16).width(16),
                                                    CalendarSpan::End => band.left(0).width(16),
                                                    CalendarSpan::Middle => band.left(0).width(32),
                                                    CalendarSpan::None => band.width(0),
                                                }
                                            });
                                            let mut day = node()
                                                .part(part)
                                                .on_pointer(move |event: PointerEvent| {
                                                    if event.kind != PointerKind::Touch {
                                                        match event.phase {
                                                            PointerPhase::Enter
                                                            | PointerPhase::Move
                                                                if pending
                                                                    .get_untracked()
                                                                    .is_some()
                                                                    && !state
                                                                        .get_untracked()
                                                                        .disabled =>
                                                            {
                                                                hovered.set(Some(date))
                                                            }
                                                            PointerPhase::Leave
                                                                if hovered.get_untracked()
                                                                    == Some(date) =>
                                                            {
                                                                hovered.set(None)
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                    Handled::No
                                                })
                                                .on_press(move || {
                                                    // A pointer can choose a day outside the current tab
                                                    // stop. Focus the new tab stop after the grid updates.
                                                    kivact::task::spawn(async move {
                                                        grid.focus_active();
                                                    });
                                                });
                                            if date == today {
                                                day = day.description(message(
                                                    "soft.calendar.today",
                                                    "Today",
                                                ));
                                            }
                                            let day = day
                                                .child(text(date.day().to_string()))
                                                .style(move || {
                                                    let t = tokens();
                                                    let state = state.get();
                                                    let selected = state.edge;
                                                    let mut day = style()
                                                        .position(Position::Absolute)
                                                        .left(2)
                                                        .top(2)
                                                        .size(28, 28)
                                                        .radius(9999)
                                                        .font_size(14)
                                                        .weight(400)
                                                        .align_items(Align::Center)
                                                        .justify_content(Justify::Center)
                                                        .color(if selected {
                                                            t.colors.on_accent
                                                        } else {
                                                            t.colors.text_primary
                                                        })
                                                        .background(if selected {
                                                            t.colors.accent
                                                        } else {
                                                            Color::TRANSPARENT
                                                        })
                                                        .focus_visible(style().ring_with_offset(
                                                            2,
                                                            3,
                                                            t.colors.accent,
                                                        ));
                                                    if date == today {
                                                        day = day.border(
                                                            1,
                                                            if state.span == CalendarSpan::None {
                                                                t.colors.border_emphasized
                                                            } else {
                                                                t.colors.text_primary
                                                            },
                                                        );
                                                    }
                                                    if state.disabled {
                                                        day.opacity(0.3)
                                                    } else if selected {
                                                        day
                                                    } else {
                                                        day.hover(
                                                            style()
                                                                .background(t.colors.overlay_hover),
                                                        )
                                                        .pressed(
                                                            style().background(
                                                                t.colors.overlay_pressed,
                                                            ),
                                                        )
                                                    }
                                                });
                                            node()
                                                .child(band)
                                                .child(day)
                                                .style(
                                                    style()
                                                        .position(Position::Relative)
                                                        .size(32, 32),
                                                )
                                                .into_view()
                                        })
                                        .collect::<Vec<_>>())
                                    .part(grid.row())
                                    .into_view()
                                })
                                .collect::<Vec<_>>();
                            column((headers, weeks))
                                .part(grid.grid())
                                .name(title(month))
                                .into_view()
                        })
                        .collect::<Vec<_>>();
                    if autofocus.replace(false) {
                        on_mount(move || {
                            grid.focus_active();
                        });
                    }
                    column((caption, row(months).style(style().gap(16).wrap(true))))
                        .style(style().gap(8))
                },
            ))
            .into_view()
    }
}
