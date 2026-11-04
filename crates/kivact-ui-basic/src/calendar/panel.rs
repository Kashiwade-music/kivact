//! Role: One month of the calendar: caption, weekday row and day cells.

use super::caption::{Names, title};
use super::state::Shared;
use crate::theme_access::{tokens, wide_focus_ring};
use kivact::custom::{Part, PartState};
use kivact::i18n::{CalendarDate, CalendarMonth, Weekday};
use kivact::interaction::{CalendarDayState, CalendarSpan};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, TextAlign, pct, style};
use kivact::ui::{ButtonVariant, ControlSize};
use kivact::view::{IntoView, Role, View, show};
use kivact::view::{Node, NodeBuilder, column, node, row, text};
use std::rc::Rc;

/// The day size of `--cell-size`.
const CELL: f64 = 28.0;

/// A ghost icon button of the caption row, 28 px square, behind a paging part.
fn nav_button(
    glyph: &'static str,
    name: kivact::state::Value<String>,
    paging: Part<PartState>,
) -> View {
    kivact::ui::button(name)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(kivact::view::icon(glyph))
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
    let rtl = kivact::view::use_direction().get() == kivact::view::ReadingDirection::Rtl;
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
                        .color(crate::Color::MutedForeground),
                )
                .into_view()
        })
        .collect::<Vec<_>>())
    .with_node(|root| root.part(grid.row()));
    let rows = month
        .weeks(first, 0)
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
    column((
        caption,
        column((headers, rows))
            .with_node(|root| root.part(grid.grid()))
            .name(format!("{}-{:02}", month.year(), month.month())),
    ))
    .style(style().gap(16).width(CELL * 7.0).shrink(0))
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
    let today = day == shared.today;
    let button = node()
        .with_node(|root| root.part(cell))
        .child(text(day.day().to_string()))
        .style(move || day_style(marked.get(), today, in_month));
    let band = node().style(move || {
        let p = tokens();
        let s = style().position(Position::Absolute).top(0).height(CELL);
        match marked.get().span {
            CalendarSpan::Start => s.left(CELL / 2.0).width(CELL / 2.0).background(p.muted),
            CalendarSpan::End => s.left(0).width(CELL / 2.0).background(p.muted),
            CalendarSpan::Middle => s.left(0).width(CELL).background(p.muted),
            CalendarSpan::None => s.width(0),
        }
    });
    node().child(band).child(button).style(
        style()
            .position(Position::Relative)
            .size(CELL, CELL)
            .shrink(0),
    )
}

fn day_style(state: CalendarDayState, today: bool, in_month: bool) -> Style {
    let p = tokens();
    let dark = p.is_dark();
    let disabled = state.disabled;
    let (fill, color) = if state.edge {
        (p.primary, p.primary_foreground)
    } else if state.span == CalendarSpan::Middle || (today && in_month) {
        (p.muted, p.foreground)
    } else if !in_month || disabled {
        (Color::TRANSPARENT, p.muted_foreground)
    } else {
        (Color::TRANSPARENT, p.foreground)
    };
    let hover = if dark {
        p.muted.with_alpha(0.5)
    } else {
        p.muted
    };
    let s = style()
        .position(Position::Absolute)
        .left(0)
        .top(0)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .size(CELL, CELL)
        .radius(0)
        .border(1, Color::TRANSPARENT)
        .font_size(12)
        .weight(400)
        .text_wrap(false)
        .background(fill)
        .color(color)
        .focus_visible(wide_focus_ring(&p));
    if state.edge || disabled {
        s
    } else {
        s.hover(style().background(hover))
    }
}
