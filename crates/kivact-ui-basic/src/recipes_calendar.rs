//! Role: Lyra calendar panels, day cells and range-band recipes.
use crate::Tokens;
use crate::theme_access::wide_focus_ring;
use kivact::interaction::{CalendarSpan, DaySelection};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, style};
use kivact::ui::{CalendarRecipes, Look};
const CELL: f64 = 28.0;
pub(crate) fn calendar(r: CalendarRecipes) -> CalendarRecipes {
    r.root(|_| root)
        .month(|_| month)
        .day(|_| day)
        .range(|_| range)
}
fn root(t: &Theme, _: &Look<kivact::custom::PartState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .gap(16)
        .padding(8)
        .shrink(0)
        .background(p.background)
        .color(p.foreground)
        .font_size(12)
        .align_self(Align::Start)
}
fn month(_: &Theme, _: &Look<kivact::custom::PartState>) -> Style {
    style().gap(16).width(CELL * 7.0).shrink(0)
}
fn range(t: &Theme, s: &Look<kivact::interaction::CalendarDayState>) -> Style {
    let base = style()
        .position(Position::Absolute)
        .top(0)
        .height(CELL)
        .background(t.ext::<Tokens>().muted);
    match s.span {
        CalendarSpan::Start => base.left(CELL / 2.0).width(CELL / 2.0),
        CalendarSpan::End => base.left(0).width(CELL / 2.0),
        CalendarSpan::Middle => base.left(0).width(CELL),
        CalendarSpan::None => base.width(0),
    }
}
fn day(t: &Theme, s: &Look<kivact::ui::CalendarDayVisual>) -> Style {
    let (state, today, in_month) = (&s.day, s.today, s.in_month);
    let p = t.ext::<Tokens>();
    let dark = p.is_dark();
    let disabled = state.disabled;
    let (fill, color) = if state.selection == DaySelection::Edge {
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
    if state.selection == DaySelection::Edge || disabled {
        s
    } else {
        s.hover(style().background(hover))
    }
}
