//! Role: Lyra calendar panels, day cells and range-band recipes.
//!
//! The looks of calendar.tsx: the panel, its month, the band of a range and
//! the day cells.

use crate::color::{BACKGROUND, FOREGROUND, MUTED, MUTED_FOREGROUND, PRIMARY, PRIMARY_FOREGROUND};
use crate::recipe_parts::wide_focus_ring;
use kivact::custom::PartState;
use kivact::interaction::{CalendarDayState, CalendarSpan, DaySelection};
use kivact::style::{
    Align, Color, FlexDirection, Justify, Position, Style, Theme, ThemeMode, style,
};
use kivact::ui::theme::{CalendarDayVisual, CalendarTheme, Look};

/// The width and height of a day cell.
const CELL: f64 = 28.0;

/// `flex gap-4 p-2`: the panel holds its months side by side.
fn calendar_root(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(16.0)
        .padding(8.0)
        .flex_shrink(0.0)
        .background(BACKGROUND)
        .color(FOREGROUND)
        .font_size(12.0)
        .align_self(Align::Start)
}

/// A month: seven cells wide, its weeks 16 px apart.
fn calendar_month(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().gap(16.0).width(CELL * 7.0).flex_shrink(0.0)
}

/// The band behind a range: half a cell at the start and the end, a full cell
/// in the middle.
fn calendar_range(_: &Theme, s: &Look<CalendarDayState>, _base: Style) -> Style {
    let base = style()
        .position(Position::Absolute)
        .top(0.0)
        .height(CELL)
        .background(MUTED);
    match s.span {
        CalendarSpan::Start => base.left(CELL / 2.0).width(CELL / 2.0),
        CalendarSpan::End => base.left(0.0).width(CELL / 2.0),
        CalendarSpan::Middle => base.left(0.0).width(CELL),
        CalendarSpan::None => base.width(0.0),
    }
}

/// A day cell: a square of `CELL`, in the primary color when selected at an
/// edge, muted in a range or on today, and muted-foreground outside the month.
fn calendar_day(t: &Theme, s: &Look<CalendarDayVisual>, _base: Style) -> Style {
    let (state, today, in_month) = (&s.day, s.today, s.in_month);
    let dark = t.mode() == ThemeMode::Dark;
    let disabled = state.disabled;
    let (fill, color) = if state.selection == DaySelection::Edge {
        (PRIMARY, PRIMARY_FOREGROUND)
    } else if state.span == CalendarSpan::Middle || (today && in_month) {
        (MUTED, FOREGROUND)
    } else if !in_month || disabled {
        (Color::TRANSPARENT, MUTED_FOREGROUND)
    } else {
        (Color::TRANSPARENT, FOREGROUND)
    };
    let hover = if dark { MUTED.with_alpha(0.5) } else { MUTED };
    let s = style()
        .position(Position::Absolute)
        .left(0.0)
        .top(0.0)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .size(CELL, CELL)
        .border_radius(0.0)
        .border(1.0, Color::TRANSPARENT)
        .font_size(12.0)
        .font_weight(400)
        .text_wrap(false)
        .background(fill)
        .color(color)
        .focus_visible(wide_focus_ring());
    if state.selection == DaySelection::Edge || disabled {
        s
    } else {
        s.hover(style().background(hover))
    }
}

/// Installs the calendar parts of the kit.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CalendarTheme::default()
            .root(calendar_root)
            .month(calendar_month)
            .day(calendar_day)
            .range(calendar_range),
    )
}
