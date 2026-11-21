//! Role: Install Astryx Neutral's recipes of the calendar group: the calendar, and the field and wheel of the date and time inputs.
use crate::color as c;
use crate::radius as r;
use crate::space as s;
use crate::type_scale as t;
use kivact::custom::PartState;
use kivact::interaction::{CalendarDayState, CalendarSpan, DaySelection};
use kivact::style::{Align, Color, FlexDirection, Justify, Position, Style, Theme, style};
use kivact::ui::theme::{CalendarDayVisual, CalendarTheme, Look, calendar};
use kivact::view::{ReadingDirection, use_direction};

/// One day of the calendar: a round cell in the accent when it is an edge of a range.
fn day(theme: &Theme, s: &Look<CalendarDayVisual>, _base: Style) -> Style {
    let size = theme.get(&calendar::CELL_SIZE_VAR);
    let inset = 2.0;
    let edge = s.day.selection == DaySelection::Edge;
    let mut base = style()
        .position(Position::Absolute)
        .left(inset)
        .top(inset)
        .size(size - inset * 2.0, size - inset * 2.0)
        .border_radius(r::RADIUS_FULL)
        .font_size(t::BODY.size_px)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .text_wrap(false)
        .color(if edge {
            c::ON_ACCENT
        } else if s.in_month {
            c::TEXT_PRIMARY
        } else {
            c::TEXT_SECONDARY
        })
        .background(if edge { c::ACCENT } else { Color::TRANSPARENT })
        .focus_visible(style().outline(2.0, c::ACCENT).outline_offset(2.0));
    if s.today {
        base = base.border(
            1.0,
            if s.day.span == CalendarSpan::None {
                c::BORDER_EMPHASIZED
            } else {
                c::TEXT_PRIMARY
            },
        );
    }
    if s.mark.is_some() {
        base = base.border_bottom(2.0, c::ACCENT);
    }
    if s.day.disabled || !s.in_month {
        base.opacity(0.3)
    } else if edge {
        base
    } else {
        base.hover(style().background(c::OVERLAY_HOVER))
            .active(style().background(c::OVERLAY_PRESSED))
    }
}

/// The band of a range behind its days, from the span's edge to the cell edge in the reading direction.
fn range(theme: &Theme, s: &Look<CalendarDayState>, _base: Style) -> Style {
    let size = theme.get(&calendar::CELL_SIZE_VAR);
    let base = style()
        .position(Position::Absolute)
        .top(2)
        .height(size - 4.0)
        .background(c::ACCENT_MUTED);
    let rtl = use_direction() == ReadingDirection::Rtl;
    match s.span {
        CalendarSpan::Start if rtl => base.left(0).width(size / 2.0),
        CalendarSpan::End if rtl => base.left(size / 2.0).width(size / 2.0),
        CalendarSpan::Start => base.left(size / 2.0).width(size / 2.0),
        CalendarSpan::End => base.left(0).width(size / 2.0),
        CalendarSpan::Middle => base.left(0).width(size),
        CalendarSpan::None => base.width(0),
    }
}

/// The calendar's root: the cells in a row, in the primary text color.
fn root(_theme: &Theme, _s: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(s::SPACE_S4)
        .color(c::TEXT_PRIMARY)
}

/// One month of the calendar: its weeks in a column.
fn month(_theme: &Theme, _s: &Look<PartState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column)
}

/// The field of a date or time input: its parts in a centered row.
pub(crate) fn field(_theme: &Theme, _s: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(s::SPACE_S2)
}

/// The wheel of a date or time input: a scrolling column of values.
pub(crate) fn wheel(_theme: &Theme, _s: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .min_width(48)
        .height(264)
        .clip(true)
        .background(c::BG_SURFACE)
        .color(c::TEXT_PRIMARY)
        .border_radius(r::RADIUS_ELEMENT)
        .font_size(17)
        .focus_visible(style().outline(2.0, c::ACCENT).outline_offset(3.0))
}

/// Install the kit's recipes of the calendar group.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CalendarTheme::default()
            .root(root)
            .month(month)
            .day(day)
            .range(range),
    )
}
