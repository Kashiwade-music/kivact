//! Role: Style canonical calendars and date/time field surfaces with Neutral tokens.
use crate::Tokens;
use kivact::custom::PartState;
use kivact::interaction::{CalendarDayState, CalendarSpan, DaySelection};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, style};
use kivact::ui::{CalendarDayVisual, CalendarRecipes, Look};
use kivact::view::{ReadingDirection, use_direction};
pub(crate) fn calendar(r: CalendarRecipes) -> CalendarRecipes {
    r.root(|_| {
        |theme: &Theme, _: &Look<PartState>| {
            let t = theme.ext::<Tokens>();
            style()
                .direction(Direction::Row)
                .gap(t.spacing.s4)
                .color(t.colors.text_primary)
        }
    })
    .month(|_| |_: &Theme, _: &Look<PartState>| style().direction(Direction::Column))
    .day(|_| day)
    .range(|_| range)
}
fn day(theme: &Theme, s: &Look<CalendarDayVisual>) -> Style {
    let t = theme.ext::<Tokens>();
    let size = theme.ext::<kivact::ui::UiTokens>().calendar.cell_size;
    let inset = 2.0;
    let mut base = style()
        .position(Position::Absolute)
        .left(inset)
        .top(inset)
        .size(size - inset * 2.0, size - inset * 2.0)
        .radius(t.radius.full)
        .font_size(t.typography.body.size_px)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .text_wrap(false)
        .color(if s.day.selection == DaySelection::Edge {
            t.colors.on_accent
        } else if s.in_month {
            t.colors.text_primary
        } else {
            t.colors.text_secondary
        })
        .background(if s.day.selection == DaySelection::Edge {
            t.colors.accent
        } else {
            Color::TRANSPARENT
        })
        .focus_visible(style().ring_with_offset(2, 2, t.colors.accent));
    if s.today {
        base = base.border(
            1,
            if s.day.span == CalendarSpan::None {
                t.colors.border_emphasized
            } else {
                t.colors.text_primary
            },
        );
    }
    if s.mark.is_some() {
        base = base.border_bottom(2, t.colors.accent);
    }
    if s.day.disabled || !s.in_month {
        base.opacity(0.3)
    } else if s.day.selection == DaySelection::Edge {
        base
    } else {
        base.hover(style().background(t.colors.overlay_hover))
            .pressed(style().background(t.colors.overlay_pressed))
    }
}
fn range(theme: &Theme, s: &Look<CalendarDayState>) -> Style {
    let t = theme.ext::<Tokens>();
    let size = theme.ext::<kivact::ui::UiTokens>().calendar.cell_size;
    let base = style()
        .position(Position::Absolute)
        .top(2)
        .height(size - 4.0)
        .background(t.colors.accent_muted);
    let rtl = use_direction().get() == ReadingDirection::Rtl;
    match s.span {
        CalendarSpan::Start if rtl => base.left(0).width(size / 2.0),
        CalendarSpan::End if rtl => base.left(size / 2.0).width(size / 2.0),
        CalendarSpan::Start => base.left(size / 2.0).width(size / 2.0),
        CalendarSpan::End => base.left(0).width(size / 2.0),
        CalendarSpan::Middle => base.left(0).width(size),
        CalendarSpan::None => base.width(0),
    }
}
pub(crate) fn field(theme: &Theme, _: &Look<PartState>) -> Style {
    let t = theme.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.spacing.s2)
}
pub(crate) fn wheel(theme: &Theme, _: &Look<PartState>) -> Style {
    let t = theme.ext::<Tokens>();
    style()
        .direction(Direction::Column)
        .grow(1)
        .min_width(48)
        .height(264)
        .clip(true)
        .background(t.colors.bg_surface)
        .color(t.colors.text_primary)
        .radius(t.radius.element)
        .font_size(17)
        .focus_visible(style().ring_with_offset(2, 3, t.colors.accent))
}
