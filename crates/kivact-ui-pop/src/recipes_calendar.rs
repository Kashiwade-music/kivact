//! Role: The Pop looks of the calendar: its caption, its days and their events.

use crate::color;
use crate::variants::{CalendarWeekdayHues, Hue};
use kivact::custom::PartState;
use kivact::i18n::Weekday;
use kivact::interaction::{CalendarDayState, DaySelection};
use kivact::style::{Color, FlexDirection, Style, Theme, style};
use kivact::ui::theme::{CalendarDayVisual, CalendarEventVisual, CalendarTheme, Look};

/// Install the kit's recipes of the calendar.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CalendarTheme::default()
            .root(calendar_root)
            .caption(calendar_caption)
            .day(calendar_day)
            .range(calendar_range)
            .events(calendar_events)
            .event(calendar_event),
    )
}

fn calendar_root(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .min_width(280)
        .padding(0)
        .background(Color::TRANSPARENT)
}

fn calendar_caption(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.font_size(16).font_weight(600)
}

/// A day: rounded, tinted when it is in the selection or today, and in the
/// weekday's hue on the weekend if the variant gives one.
fn calendar_day(_theme: &Theme, s: &Look<CalendarDayVisual>, base: Style) -> Style {
    let mut result = base
        .border_radius(8)
        .font_size(14)
        .hover(style().background(color::SURFACE_ACTIVE));
    if s.day.selection == DaySelection::Edge {
        return result.background(color::BLUE).color(Color::WHITE);
    }
    result = result.background(if s.day.selection != DaySelection::None {
        Hue::Blue.colors().0
    } else if s.today {
        color::SURFACE_ACTIVE
    } else {
        Color::TRANSPARENT
    });
    let hue = s
        .variant::<CalendarWeekdayHues>()
        .and_then(|hues| hues.0[s.date.weekday().days_since(Weekday::Monday)]);
    result.color(if !s.in_month {
        color::TEXT_FAINT
    } else if let Some(hue) = hue {
        hue.colors().1
    } else {
        color::TEXT
    })
}

fn calendar_range(_theme: &Theme, _: &Look<CalendarDayState>, base: Style) -> Style {
    base.background(Hue::Blue.colors().0)
}

fn calendar_events(_theme: &Theme, _: &Look<PartState>, base: Style) -> Style {
    base.bottom(4)
}

fn calendar_event(_theme: &Theme, s: &Look<CalendarEventVisual>, _base: Style) -> Style {
    style()
        .size(4, 4)
        .border_radius(2)
        .background(if s.selected { Color::WHITE } else { s.color })
}
