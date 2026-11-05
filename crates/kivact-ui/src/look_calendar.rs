//! Role: Theme recipes of the calendar and date/time input surfaces.
use crate::{CalendarDayVisual, UiTokens};
use kivact_core::custom::PartState;
use kivact_core::style::{
    Align, Color, ColorToken, Direction, Justify, Position, Style, Theme, style,
};
use kivact_interaction::{CalendarDayState, CalendarSpan};
pub(super) fn root(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .gap(t.ext::<UiTokens>().calendar.month_gap)
        .padding(t.space().sm)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
}
pub(super) fn month(_: &Theme, _: &PartState) -> Style {
    style()
}
pub(super) fn day(t: &Theme, s: &CalendarDayVisual) -> Style {
    let size = t.ext::<UiTokens>().calendar.cell_size;
    let base = style()
        .position(Position::Absolute)
        .left(0)
        .top(0)
        .size(size, size)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .radius(t.radius().sm)
        .font_size(t.text().size_small)
        .text_wrap(false)
        .focus_visible(style().ring(t.focus_ring(), ColorToken::Focus))
        .disabled(style().opacity(t.disabled_opacity()));
    let base = if s.day.edge {
        base.background(ColorToken::Accent)
            .color(ColorToken::AccentText)
    } else if s.today || s.day.span == CalendarSpan::Middle {
        base.background(ColorToken::SurfaceAlt)
            .color(ColorToken::Text)
    } else {
        base.background(Color::TRANSPARENT)
            .color(if s.in_month {
                ColorToken::Text
            } else {
                ColorToken::TextMuted
            })
            .hover(style().background(ColorToken::SurfaceAlt))
    };
    if s.mark.is_some() {
        base.border_bottom(2, ColorToken::Accent)
    } else {
        base
    }
}
pub(super) fn range(t: &Theme, s: &CalendarDayState) -> Style {
    let size = t.ext::<UiTokens>().calendar.cell_size;
    let base = style()
        .position(Position::Absolute)
        .top(0)
        .height(size)
        .background(ColorToken::SurfaceAlt);
    match s.span {
        CalendarSpan::Start => base.left(size / 2.0).width(size / 2.0),
        CalendarSpan::Middle => base.left(0).width(size),
        CalendarSpan::End => base.left(0).width(size / 2.0),
        CalendarSpan::None => base.width(0),
    }
}
pub(super) fn input(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().xs)
}
pub(super) fn wheel(t: &Theme, _: &PartState) -> Style {
    style()
        .width(t.ext::<UiTokens>().date_input.wheel_width)
        .padding(t.space().sm)
        .gap(t.space().xs)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
}

pub(super) fn events(_: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .position(Position::Absolute)
        .bottom(2)
        .gap(2)
}
pub(super) fn event(_: &Theme, s: &crate::CalendarEventVisual) -> Style {
    style().size(3, 3).radius(2).background(s.color)
}
