//! Role: One month of the calendar: caption, weekday row and day cells.

use super::caption::{Names, title};
use super::state::Shared;
use crate::layer::with_recipe_style;
use crate::variants::Variants;
use kivact_core::custom::{Part, PartState};
use kivact_core::i18n::{CalendarDate, CalendarMonth, Weekday};
use kivact_interaction::{CalendarDayState, CalendarSpan};
use kivact_core::style::{Align, Color, Direction, Justify, Position, Style, TextAlign, pct, style};
use crate::{ButtonVariant, ControlSize};
use kivact_core::view::{IntoView, Role, View, show};
use kivact_core::view::{Node, NodeBuilder, column, node, row, text};
use std::rc::Rc;

/// The day size of `--cell-size`.
const CELL: f64 = 28.0;

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
    node().style(style().size(CELL, CELL).shrink(0))
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
    let caption = row((
        previous,
        node().child(title(shared, names, index, month)).style(
            style()
                .grow(1)
                .direction(Direction::Row)
                .justify_content(Justify::Center)
                .align_items(Align::Center),
        ),
        next,
    ))
    .style(
        style()
            .direction(if rtl {
                Direction::RowReverse
            } else {
                Direction::Row
            })
            .align_items(Align::Center)
            .height(CELL)
            .width(pct(100)),
    );
    let headers = row((0..7)
        .map(|c| {
            text(names.weekday(first.days_after(c)))
                .with_node(|root| root.role(Role::ColumnHeader))
                .style(
                    style()
                        .width(CELL)
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
            .style(style().margin_top(8))
            .into_view()
        })
        .collect::<Vec<_>>();
    with_recipe_style(column((
        caption,
        column((headers, rows))
            .with_node(|root| root.part(grid.grid()))
            .name(format!("{}-{:02}", month.year(), month.month())),
    ))
    .style(style().gap(16).width(CELL * 7.0).shrink(0)), |r| r.calendar_recipes().month_recipe(), &Variants::default(), PartState::default)
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
    let marked = cell.state();
    let in_month = day.month_of() == month;
    let today = shared.today;
    let marks = shared.marks.clone();
    let button = with_recipe_style(node()
        .with_node(|root| root.part(cell))
        .child(text(day.day().to_string()))
        , |r| r.calendar_recipes().day_recipe(), &Variants::default(), move || crate::CalendarDayVisual {
            day: marked.get(), today: day == today.get(), in_month, mark: marks(day),
        });
    let band = with_recipe_style(node(), |r| r.calendar_recipes().range_recipe(), &Variants::default(), move || marked.get());
    node().child(band).child(button).style(
        style()
            .position(Position::Relative)
            .size(CELL, CELL)
            .shrink(0),
    )
}

