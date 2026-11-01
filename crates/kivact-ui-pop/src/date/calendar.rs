//! Role: Render calendar dates, navigation and day selection.
use crate::{Hue, Tokens};
use kivact::control::{Node, NodeBuilder, grid, icon, node, row, text};
use kivact::i18n::{CalendarDate, CalendarMonth, Weekday, message, message_with};
use kivact::interaction::{CalendarPolicy, CalendarSelection, use_calendar_grid};
use kivact::primitive::{NodeExt, Part, PartState};
use kivact::style::{Align, Color, ColorToken, Direction, Justify, Track, current_theme, style};
use kivact::ui::{ButtonVariant, button};
use kivact::view::HandlerResult;
use kivact::view::{IntoValue, IntoView, View, callback, show};
use kivact::{Binding, Callback, Value, signal};
use std::rc::Rc;

/// One event dot; event contents remain in the application's scheduling model.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    pub date: CalendarDate,
    pub color: Color,
}

/// How a day cell relates to the selected date and the range.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    /// Neither the selected date nor inside the range.
    None,
    /// Inside the range, between its ends.
    Interior,
    /// The selected date or an end of the range.
    Solid,
}

/// A month calendar with day selection. Build it with [`calendar`].
///
/// The first column is [`week_starts_on`](Calendar::week_starts_on), by
/// default the first day of the week of the App's locale. The texts are
/// read through message keys: `calendar.sun` through `calendar.sat` for the
/// weekday headings, `calendar.previous` (`Previous month`) and
/// `calendar.next` (`Next month`) and, with
/// [`year_navigation`](Calendar::year_navigation), `calendar.previous_year`
/// (`Previous year`) and `calendar.next_year` (`Next year`) for the buttons,
/// and `calendar.title` (`{year}-{month}`, the month in two digits) for the
/// title above the days.
///
/// Keys on a focused day: the arrows move by a day or a week, Home and End
/// go to the first and last day of the week, PageUp and PageDown move a
/// month and Shift with them a year.
///
/// ```no_run
/// use kivact::i18n::{CalendarDate, Weekday};
/// use kivact::prelude::*;
/// use kivact_ui_pop as ui;
///
/// fn picker() -> impl IntoView {
///     let day = signal(CalendarDate::new(2026, 10, 3));
///     ui::calendar(day, CalendarDate::new(2026, 10, 3))
///         .week_starts_on(Weekday::Monday)
///         .year_navigation(true)
/// }
/// ```
pub struct Calendar {
    root: Node,
    selected: Binding<CalendarDate>,
    today: CalendarDate,
    events: Option<Value<Vec<CalendarEvent>>>,
    on_activate: Option<Callback<CalendarDate>>,
    week_starts_on: Option<Value<Weekday>>,
    year_navigation: Value<bool>,
    year_month_menus: Value<bool>,
    month_title: Option<Rc<dyn Fn(CalendarDate) -> String>>,
    range: Option<Value<Option<(CalendarDate, CalendarDate)>>>,
    weekend: Rc<dyn Fn(Weekday) -> Option<Hue>>,
}

/// A month calendar over `selected`. The application supplies `today`.
#[track_caller]
pub fn calendar(selected: impl Into<Binding<CalendarDate>>, today: CalendarDate) -> Calendar {
    Calendar {
        root: node().style(style().direction(Direction::Column).min_width(280)),
        selected: selected.into(),
        today,
        events: None,
        on_activate: None,
        week_starts_on: None,
        year_navigation: false.into_value(),
        year_month_menus: false.into_value(),
        month_title: None,
        range: None,
        weekend: Rc::new(|_| None),
    }
}

impl Calendar {
    /// Optional application-defined day tones; by default all weekdays share a hue.
    pub fn weekend(mut self, weekend: impl Fn(Weekday) -> Option<Hue> + 'static) -> Self {
        self.weekend = Rc::new(weekend);
        self
    }
    /// Up to three application-provided event dots per date.
    pub fn events(mut self, events: impl IntoValue<Vec<CalendarEvent>>) -> Self {
        self.events = Some(events.into_value());
        self
    }

    /// Called with the date of an explicit pointer or Enter activation,
    /// including the date that is already selected.
    pub fn on_activate<R: HandlerResult>(
        mut self,
        f: impl Fn(CalendarDate) -> R + 'static,
    ) -> Self {
        self.on_activate = Some(callback(f));
        self
    }

    /// The first column of the week (default: the locale's first day of the
    /// week).
    pub fn week_starts_on(mut self, day: impl IntoValue<Weekday>) -> Self {
        self.week_starts_on = Some(day.into_value());
        self
    }

    /// Whether to show buttons that move a year back and forward outside the
    /// month buttons (default `false`; constant or reactive). Shift with
    /// PageUp and PageDown move a year either way.
    #[track_caller]
    pub fn year_navigation(mut self, on: impl IntoValue<bool>) -> Self {
        self.year_navigation = on.into_value();
        self
    }

    /// Replace the title with year and month selects (default `false`;
    /// constant or reactive).
    /// The year menu covers the shown year plus or minus 50. The labels use
    /// `calendar.year` (`Year`) and `calendar.month` (`Month`); each month's
    /// label uses the same formatter as [`Self::month_title`]. Navigation
    /// changes the shown month without changing the selected day.
    #[track_caller]
    pub fn year_month_menus(mut self, on: impl IntoValue<bool>) -> Self {
        self.year_month_menus = on.into_value();
        self
    }

    /// The title above the days, from the first day of the shown month
    /// (default: the message `calendar.title`).
    pub fn month_title(mut self, f: impl Fn(CalendarDate) -> String + 'static) -> Self {
        self.month_title = Some(Rc::new(f));
        self
    }

    /// Fill the range of dates, both ends included. Every day in it is
    /// `selected` for assistive technology, and its ends are drawn like the
    /// selected date.
    pub fn range(mut self, range: impl IntoValue<Option<(CalendarDate, CalendarDate)>>) -> Self {
        self.range = Some(range.into_value());
        self
    }
}

impl NodeBuilder for Calendar {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// The column of `day` in a week that starts on Sunday (0) through Saturday (6).
fn sunday_zero(day: Weekday) -> usize {
    match day {
        Weekday::Sunday => 0,
        Weekday::Monday => 1,
        Weekday::Tuesday => 2,
        Weekday::Wednesday => 3,
        Weekday::Thursday => 4,
        Weekday::Friday => 5,
        Weekday::Saturday => 6,
    }
}

impl IntoView for Calendar {
    fn into_view(self) -> View {
        let Calendar {
            root,
            selected,
            today,
            events,
            on_activate,
            week_starts_on,
            year_navigation,
            year_month_menus,
            month_title,
            range,
            weekend,
        } = self;
        let value = selected.into_value();
        let visible = signal(selected.get_untracked().month_of());
        kivact::effect(move || {
            let month = value.get().month_of();
            if month != visible.get_untracked() {
                visible.set(month);
            }
        });
        let mut policy = CalendarPolicy::new().select_on_move(true);
        if let Some(day) = week_starts_on {
            policy = policy.week_start(day);
        }
        if let Some(on_activate) = on_activate {
            policy = policy.on_commit(move |date| on_activate.call(date));
        }
        let choice = kivact::binding(
            move || Some(value.get()),
            move |date: Option<CalendarDate>| {
                if let Some(date) = date {
                    selected.set(date);
                }
            },
        );
        let behavior = use_calendar_grid(visible, CalendarSelection::Single(choice), policy);
        let first_day = behavior.week_start();
        let events = events.unwrap_or_else(|| (Vec::new()).into_value());
        let day_names = [
            message("pop.calendar.sun", "Sun"),
            message("pop.calendar.mon", "Mon"),
            message("pop.calendar.tue", "Tue"),
            message("pop.calendar.wed", "Wed"),
            message("pop.calendar.thu", "Thu"),
            message("pop.calendar.fri", "Fri"),
            message("pop.calendar.sat", "Sat"),
        ];
        let headers = (0..7)
            .map(|index| {
                text(move || day_names[sunday_zero(first_day.get().days_after(index))].get())
                    .style(
                        style()
                            .font_size(12)
                            .text_align(kivact::style::TextAlign::Center)
                            .color(ColorToken::TextMuted)
                            .padding_y(4),
                    )
                    .into_view()
            })
            .collect::<Vec<_>>();
        let shown = kivact::memo(move || visible.get().first_day());
        let title = month_label(shown, month_title.clone());
        let step = move |glyph: &'static str, label: Value<String>, paging: Part<PartState>| {
            button(label)
                .variant(ButtonVariant::Ghost)
                .hide_label(true)
                .icon(icon(glyph))
                .part(paging)
                .into_view()
        };
        let mut bar = Vec::new();
        bar.push(
            show(year_navigation, move || {
                let label = message("pop.calendar.previous_year", "Previous year");
                step("chevrons-left", label, behavior.previous_year())
            })
            .into_view(),
        );
        bar.push(step(
            "chevron-left",
            message("pop.calendar.previous", "Previous month"),
            behavior.previous_month(),
        ));
        let menus = {
            let month_title = month_title.clone();
            move || {
                use kivact::ui::ListItem;
                let years = move || {
                    let year = visible.get().year();
                    (year.saturating_sub(50)..=year.saturating_add(50))
                        .map(|year| ListItem::new(year, year.to_string()))
                        .collect::<Vec<_>>()
                };
                let year = kivact::binding(
                    move || Some(visible.get().year()),
                    move |year| {
                        if let Some(year) = year {
                            visible
                                .update(|month| *month = CalendarMonth::new(year, month.month()));
                        }
                    },
                );
                let months = (1..=12)
                    .map(|month| {
                        let date =
                            kivact::memo(move || CalendarDate::new(visible.get().year(), month, 1));
                        ListItem::new(month, month_label(date, month_title.clone()))
                    })
                    .collect::<Vec<_>>();
                let month = kivact::binding(
                    move || Some(visible.get().month()),
                    move |month| {
                        if let Some(month) = month {
                            visible
                                .update(|shown| *shown = CalendarMonth::new(shown.year(), month));
                        }
                    },
                );
                (
                    kivact::ui::select(message("pop.calendar.year", "Year"), years, year),
                    kivact::ui::select(message("pop.calendar.month", "Month"), months, month),
                )
            }
        };
        let heading = move || {
            text(title).style(
                style()
                    .font_size(16)
                    .weight(600)
                    .grow(1)
                    .text_align(kivact::style::TextAlign::Center),
            )
        };
        bar.push(show(year_month_menus, menus).fallback(heading).into_view());
        bar.push(step(
            "chevron-right",
            message("pop.calendar.next", "Next month"),
            behavior.next_month(),
        ));
        bar.push(
            show(year_navigation, move || {
                let label = message("pop.calendar.next_year", "Next year");
                step("chevrons-right", label, behavior.next_year())
            })
            .into_view(),
        );
        let days = kivact::view::dynamic(
            move || (visible.get(), first_day.get()),
            move |(month, first)| {
                grid(
                    month
                        .weeks(first, 6)
                        .into_iter()
                        .flatten()
                        .map(|date| {
                            // Weekend colors follow the weekday, not the column.
                            let weekday = date.weekday();
                            let weekend = weekend.clone();
                            let cell = behavior.cell(month, date);
                            let state = cell.state();
                            let mark = move || {
                                let cursor = state.get().selected;
                                match range.and_then(|range| range.get()) {
                                    Some((start, end)) if start <= date && date <= end => {
                                        if cursor || date == start || date == end {
                                            Mark::Solid
                                        } else {
                                            Mark::Interior
                                        }
                                    }
                                    _ if cursor => Mark::Solid,
                                    _ => Mark::None,
                                }
                            };
                            node()
                                .part(cell)
                                .selected(move || mark() != Mark::None)
                                .style(move || {
                                    let t = &current_theme();
                                    let p = t.get::<Tokens>();
                                    let mark = mark();
                                    style()
                                        .height(40)
                                        .direction(Direction::Row)
                                        .align_items(Align::Center)
                                        .justify_content(Justify::Center)
                                        .radius(8)
                                        .background(if mark == Mark::Interior {
                                            p.hue(Hue::Blue).0
                                        } else if date == today {
                                            p.surface_active
                                        } else {
                                            Color::TRANSPARENT
                                        })
                                        .color(if date.month_of() != month {
                                            p.text_faint
                                        } else if let Some(hue) = weekend(weekday) {
                                            p.hue(hue).1
                                        } else {
                                            p.text
                                        })
                                        .hover(style().background(p.surface_active))
                                        .selected(if mark == Mark::Interior {
                                            style().background(p.hue(Hue::Blue).0)
                                        } else {
                                            style().background(p.blue).color(Color::WHITE)
                                        })
                                })
                                .child(text(date.day().to_string()))
                                .draw(move |draw, size| {
                                    let colors = events.with(|list| {
                                        list.iter()
                                            .filter(|event| event.date == date)
                                            .take(3)
                                            .map(|event| event.color)
                                            .collect::<Vec<_>>()
                                    });
                                    let count = colors.len();
                                    for (index, color) in colors.into_iter().enumerate() {
                                        draw.rounded_rect(
                                            kivact::draw::Rect::new(
                                                (size.width - (count as f32 * 6.0 - 2.0)) / 2.0
                                                    + index as f32 * 6.0,
                                                size.height - 8.0,
                                                4.0,
                                                4.0,
                                            ),
                                            2.0,
                                            if mark() == Mark::Solid {
                                                Color::WHITE
                                            } else {
                                                color
                                            },
                                        );
                                    }
                                })
                                .into_view()
                        })
                        .collect::<Vec<_>>(),
                )
                .part(behavior.grid())
                .style(
                    style()
                        .grid_template_columns(Track::repeat(7, Track::fr(1)))
                        .gap(2),
                )
                .into_view()
            },
        );
        root.child(row(bar).style(style().align_items(Align::Center).margin_bottom(12)))
            .child(
                grid(headers).style(style().grid_template_columns(Track::repeat(7, Track::fr(1)))),
            )
            .child(days)
            .into_view()
    }
}

fn month_label(
    date: Value<CalendarDate>,
    formatter: Option<Rc<dyn Fn(CalendarDate) -> String>>,
) -> Value<String> {
    match formatter {
        Some(formatter) => (move || formatter(date.get())).into_value(),
        None => message_with("pop.calendar.title", "{year}-{month}", move || {
            let date = date.get();
            [
                ("year", date.year().to_string()),
                ("month", format!("{:02}", date.month())),
            ]
        }),
    }
}
