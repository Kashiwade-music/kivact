//! Role: One month of the calendar: caption, weekday row and day cells.

use super::caption::{Names, title};
use super::state::Shared;
use crate::layer::with_recipe_style;
use crate::{ButtonVariant, ControlSize};
use kivact_core::custom::{Part, PartState};
use kivact_core::i18n::{CalendarDate, CalendarMonth, Weekday};
use kivact_core::style::{Align, Direction, Justify, Position, TextAlign, pct, style};
use kivact_core::view::{IntoView, Role, View, show};
use kivact_core::view::{Node, NodeBuilder, column, node, row, text};
use std::rc::Rc;

/// The day size of `--cell-size`.
fn cell_size() -> f64 {
    kivact_core::style::use_theme()
        .get()
        .ext::<crate::UiTokens>()
        .calendar
        .cell_size
}

/// A ghost icon button of the caption row, 28 px square, behind a paging part.
fn nav_button(
    glyph: &'static str,
    name: kivact_core::state::Value<String>,
    paging: Part<PartState>,
) -> View {
    crate::button(name)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(kivact_core::view::icon(glyph))
        .size(ControlSize::Small)
        .with_node(|root| root.part(paging))
        .into_view()
}

fn spacer() -> Node {
    node().style(style().size(cell_size(), cell_size()).shrink(0))
}

pub(super) fn month_panel(
    shared: &Rc<Shared>,
    names: &Rc<Names>,
    index: usize,
    month: CalendarMonth,
    first: Weekday,
) -> Node {
    let rtl = kivact_core::view::use_direction().get() == kivact_core::view::ReadingDirection::Rtl;
    let (back, forward) = if rtl {
        ("chevron-right", "chevron-left")
    } else {
        ("chevron-left", "chevron-right")
    };
    let grid = shared.grid;
    let previous = if index == 0 {
        nav_button(back, names.previous, grid.previous_month())
    } else {
        spacer().into_view()
    };
    let next = if index + 1 == shared.count {
        nav_button(forward, names.next, grid.next_month())
    } else {
        spacer().into_view()
    };
    let previous_year = show(shared.year_navigation, move || {
        nav_button(
            "chevrons-left",
            kivact_core::i18n::message("calendar.previous_year", "Previous year"),
            grid.previous_year(),
        )
    });
    let next_year = show(shared.year_navigation, move || {
        nav_button(
            "chevrons-right",
            kivact_core::i18n::message("calendar.next_year", "Next year"),
            grid.next_year(),
        )
    });
    let caption = row((
        previous_year,
        previous,
        node().child(title(shared, names, index, month)).style(
            style()
                .grow(1)
                .direction(Direction::Row)
                .justify_content(Justify::Center)
                .align_items(Align::Center),
        ),
        next,
        next_year,
    ))
    .style(
        style()
            .direction(if rtl {
                Direction::RowReverse
            } else {
                Direction::Row
            })
            .align_items(Align::Center)
            .height(cell_size())
            .width(pct(100)),
    );
    let caption = with_recipe_style(
        caption,
        |r| r.calendar_recipes().caption_recipe(),
        &shared.variants,
        PartState::default,
    );
    let locale = kivact_core::i18n::use_locale();
    let headers = row((0..7)
        .map(|c| {
            node()
                .child(text(names.weekday(first.days_after(c))))
                .with_node(|root| root.role(Role::ColumnHeader))
                .name(move || {
                    locale
                        .get()
                        .weekday_name(first.days_after(c), kivact_core::i18n::NameWidth::Long)
                })
                .style(
                    style()
                        .width(cell_size())
                        .height(20)
                        .font_size(12.8)
                        .text_align(TextAlign::Center)
                        .color(kivact_core::style::ColorToken::TextMuted),
                )
                .into_view()
        })
        .collect::<Vec<_>>())
    .with_node(|root| root.part(grid.row()));
    let rows = month
        .weeks(first, if shared.variable_rows.get() { 0 } else { 6 })
        .into_iter()
        .map(|week| {
            row(week
                .into_iter()
                .map(|day| day_cell(shared, month, day))
                .collect::<Vec<_>>())
            .with_node(|root| root.part(grid.row()))
            .style(
                style().margin_top(
                    kivact_core::style::use_theme()
                        .get()
                        .ext::<crate::UiTokens>()
                        .calendar
                        .week_gap,
                ),
            )
            .into_view()
        })
        .collect::<Vec<_>>();
    with_recipe_style(
        column((
            caption,
            column((headers, rows))
                .with_node(|root| root.part(grid.grid()))
                .name(format!("{}-{:02}", month.year(), month.month())),
        ))
        .style(
            style()
                .gap(
                    kivact_core::style::use_theme()
                        .get()
                        .ext::<crate::UiTokens>()
                        .calendar
                        .month_gap,
                )
                .width(cell_size() * 7.0)
                .shrink(0),
        ),
        |r| r.calendar_recipes().month_recipe(),
        &shared.variants,
        PartState::default,
    )
}

/// One day: a band behind a button, as calendar.tsx draws a range.
fn day_cell(shared: &Rc<Shared>, month: CalendarMonth, day: CalendarDate) -> View {
    if day.month_of() == month {
        return day_node(shared, month, day).into_view();
    }
    let shared = shared.clone();
    show(shared.outside, move || day_node(&shared, month, day))
        .fallback(spacer)
        .into_view()
}

fn day_node(shared: &Rc<Shared>, month: CalendarMonth, day: CalendarDate) -> Node {
    let cell = shared.grid.cell(month, day);
    let observed = cell.state();
    let range = shared.range;
    let marked = kivact_core::state::memo(move || {
        let mut state = observed.get();
        if let Some(range) = range.get() {
            if day >= range.start() && day <= range.end() {
                state.selected = true;
                state.edge = day == range.start() || day == range.end();
                state.span = if range.start() == range.end() {
                    kivact_interaction::CalendarSpan::None
                } else if day == range.start() {
                    kivact_interaction::CalendarSpan::Start
                } else if day == range.end() {
                    kivact_interaction::CalendarSpan::End
                } else {
                    kivact_interaction::CalendarSpan::Middle
                };
            }
        }
        state
    });
    let in_month = day.month_of() == month;
    let today = shared.today;
    let marks = shared.marks.clone();
    let weekend = shared.weekend.clone();
    let events = shared.events;
    let button = with_recipe_style(
        node()
            .with_node(|root| root.part(cell))
            .selected(move || marked.get().selected)
            .child(text(day.day().to_string()))
            .child(kivact_core::view::dynamic(
                move || {
                    events
                        .get()
                        .into_iter()
                        .filter(|event| event.date == day)
                        .take(3)
                        .collect::<Vec<_>>()
                },
                move |events| {
                    row(events
                        .into_iter()
                        .map(|event| {
                            node().style(style().size(3, 3).radius(2).background(event.color))
                        })
                        .collect::<Vec<_>>())
                    .style(style().position(Position::Absolute).bottom(2).gap(2))
                },
            )),
        |r| r.calendar_recipes().day_recipe(),
        &shared.variants,
        move || crate::CalendarDayVisual {
            date: day,
            day: marked.get(),
            today: day == today.get(),
            in_month,
            mark: marks(day).or_else(|| weekend(day.weekday())),
        },
    );
    let band = with_recipe_style(
        node(),
        |r| r.calendar_recipes().range_recipe(),
        &shared.variants,
        move || marked.get(),
    );
    node().child(band).child(button).style(
        style()
            .position(Position::Relative)
            .size(cell_size(), cell_size())
            .shrink(0),
    )
}
