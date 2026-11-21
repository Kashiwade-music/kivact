//! Role: The calendar recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::i18n::CalendarDate;
use kivact_core::style::{Align, Color, FlexDirection, Justify, Position, Style, Theme, style};
use kivact_core::style::{Length, ThemeVar};
use kivact_interaction::{CalendarDayState, CalendarSpan};

/// The side of a calendar's day cell, in logical px.
pub static CELL_SIZE_VAR: ThemeVar<f64> = ThemeVar::new("calendar.cell_size", 32.0);
/// [`CELL_SIZE_VAR`] as a length for a style.
pub const CELL_SIZE: Length = Length::var(&CELL_SIZE_VAR);
/// The gap between the months of a calendar, in logical px.
pub static MONTH_GAP_VAR: ThemeVar<f64> = ThemeVar::new("calendar.month_gap", 16.0);
/// [`MONTH_GAP_VAR`] as a length for a style.
pub const MONTH_GAP: Length = Length::var(&MONTH_GAP_VAR);
/// The gap between the weeks of a month, in logical px.
pub static WEEK_GAP_VAR: ThemeVar<f64> = ThemeVar::new("calendar.week_gap", 8.0);
/// [`WEEK_GAP_VAR`] as a length for a style.
pub const WEEK_GAP: Length = Length::var(&WEEK_GAP_VAR);

pub(crate) fn root(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(MONTH_GAP)
        .padding(t.get(&kivact_core::style::space::SM_VAR))
        .background(kivact_core::style::color::SURFACE)
        .color(kivact_core::style::color::TEXT)
}

pub(crate) fn month(_: &Theme, _: &Look<PartState>) -> Style {
    style()
}

pub(crate) fn day(t: &Theme, s: &Look<CalendarDayVisual>) -> Style {
    let size = s.cell_size;
    let base = style()
        .position(Position::Absolute)
        .left(0)
        .top(0)
        .size(size, size)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .text_wrap(false)
        .focus_visible(style().outline(t.focus_ring(), kivact_core::style::color::FOCUS))
        .disabled(style().opacity(t.get(&kivact_core::style::control::DISABLED_OPACITY_VAR)));
    let base = if s.day.selection == kivact_interaction::DaySelection::Edge {
        base.background(kivact_core::style::color::ACCENT)
            .color(kivact_core::style::color::ACCENT_TEXT)
    } else if s.today || s.day.span == CalendarSpan::Middle {
        base.background(kivact_core::style::color::SURFACE_ALT)
            .color(kivact_core::style::color::TEXT)
    } else {
        base.background(Color::TRANSPARENT)
            .color(if s.in_month {
                kivact_core::style::color::TEXT
            } else {
                kivact_core::style::color::TEXT_MUTED
            })
            .hover(style().background(kivact_core::style::color::SURFACE_ALT))
    };
    if s.mark.is_some() {
        base.border_bottom(2, kivact_core::style::color::ACCENT)
    } else {
        base
    }
}

pub(crate) fn range(t: &Theme, s: &Look<CalendarDayState>) -> Style {
    let size = t.get(&CELL_SIZE_VAR);
    let base = style()
        .position(Position::Absolute)
        .top(0)
        .height(size)
        .background(kivact_core::style::color::SURFACE_ALT);
    match s.span {
        CalendarSpan::Start => base.left(size / 2.0).width(size / 2.0),
        CalendarSpan::Middle => base.left(0).width(size),
        CalendarSpan::End => base.left(0).width(size / 2.0),
        CalendarSpan::None => base.width(0),
    }
}

pub(crate) fn events(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .position(Position::Absolute)
        .bottom(2)
        .gap(2)
}

pub(crate) fn event(_: &Theme, s: &Look<CalendarEventVisual>) -> Style {
    style().size(3, 3).border_radius(2).background(s.color)
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

/// Passive drawing projection for one calendar event mark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarEventVisual {
    pub color: kivact_core::style::Color,
    pub selected: bool,
}

recipe_table! {
    /// Calendar panels and date cells.
    CalendarTheme {
        root(root_recipe): PartState = root,
        month(month_recipe): PartState = month,
        caption(caption_recipe): PartState = month,
        events(events_recipe): PartState = events,
        event(event_recipe): CalendarEventVisual = event,
        day(day_recipe): CalendarDayVisual = day,
        range(range_recipe): kivact_interaction::CalendarDayState = range,
    }
}
