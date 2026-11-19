//! Role: The calendar recipe state, tokens, recipe table and default looks.

use crate::UiTokens;
use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::i18n::CalendarDate;
use kivact_core::style::{
    Align, Color, ColorToken, Direction, Justify, Position, Style, Theme, style,
};
use kivact_interaction::{CalendarDayState, CalendarSpan};
pub(crate) fn root(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .gap(t.ext::<UiTokens>().calendar.month_gap)
        .padding(t.space().sm)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
}

pub(crate) fn month(_: &Theme, _: &PartState) -> Style {
    style()
}

pub(crate) fn day(t: &Theme, s: &CalendarDayVisual) -> Style {
    let size = s.cell_size;
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
    let base = if s.day.selection == kivact_interaction::DaySelection::Edge {
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

pub(crate) fn range(t: &Theme, s: &CalendarDayState) -> Style {
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

pub(crate) fn events(_: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .position(Position::Absolute)
        .bottom(2)
        .gap(2)
}

pub(crate) fn event(_: &Theme, s: &CalendarEventVisual) -> Style {
    style().size(3, 3).radius(2).background(s.color)
}

/// The logical state supplied to a calendar day recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarDayVisual {
    pub cell_size: f64,
    /// Date represented by this recipe projection.
    pub date: CalendarDate,
    pub day: kivact_interaction::CalendarDayState,
    pub today: bool,
    pub in_month: bool,
    pub mark: Option<kivact_interaction::Tone>,
}

/// Theme measurements of calendar dates and panels.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarTokens {
    pub cell_size: f64,
    pub month_gap: f64,
    pub week_gap: f64,
}

impl Default for CalendarTokens {
    fn default() -> Self {
        Self {
            cell_size: 32.0,
            month_gap: 16.0,
            week_gap: 8.0,
        }
    }
}

/// Passive drawing projection for one calendar event mark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarEventVisual {
    pub color: kivact_core::style::Color,
    pub selected: bool,
}

recipe_table! {
    /// Calendar panels and date cells.
    CalendarRecipes {
        root(root_recipe): PartState = root,
        month(month_recipe): PartState = month,
        caption(caption_recipe): PartState = month,
        events(events_recipe): PartState = events,
        event(event_recipe): CalendarEventVisual = event,
        day(day_recipe): CalendarDayVisual = day,
        range(range_recipe): kivact_interaction::CalendarDayState = range,
    }
}
