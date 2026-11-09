//! Role: Style standard calendars and date fields with Pop colors.
use crate::{Hue, Tokens};
use kivact::custom::PartState;
use kivact::i18n::Weekday;
use kivact::interaction::DaySelection;
use kivact::style::{Color, Direction, Theme, style};
use kivact::ui::{
    CalendarDayVisual, CalendarRecipes, DateInputRecipes, DateRangeInputRecipes, Look, Variant,
};
use std::rc::Rc;

/// Application weekday tones that retain every Pop hue.
#[derive(Clone)]
pub struct WeekendHue(Rc<dyn Fn(Weekday) -> Option<Hue>>);
impl WeekendHue {
    pub fn new(f: impl Fn(Weekday) -> Option<Hue> + 'static) -> Self {
        Self(Rc::new(f))
    }
}
impl PartialEq for WeekendHue {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl std::fmt::Debug for WeekendHue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WeekendHue(..)")
    }
}
impl Variant for WeekendHue {}

pub(crate) fn calendar(r: CalendarRecipes) -> CalendarRecipes {
    r.root(|_| {
        |_: &Theme, _: &Look<PartState>| {
            style()
                .direction(Direction::Column)
                .min_width(280)
                .padding(0)
                .background(Color::TRANSPARENT)
        }
    })
    .caption(|prev| move |t: &Theme, s: &_| prev(t, s).font_size(16).weight(600))
    .day(|prev| {
        move |t: &Theme, s: &Look<CalendarDayVisual>| {
            let p = t.ext::<Tokens>();
            let mut result = prev(t, s)
                .radius(8)
                .font_size(14)
                .hover(style().background(p.surface_active));
            if s.day.selection == DaySelection::Edge {
                return result.background(p.blue).color(Color::WHITE);
            }
            result = result.background(if s.day.selection != DaySelection::None {
                p.hue(Hue::Blue).0
            } else if s.today {
                p.surface_active
            } else {
                Color::TRANSPARENT
            });
            let hue = s
                .variant::<WeekendHue>()
                .and_then(|weekend| (weekend.0)(s.date.weekday()));
            result.color(if !s.in_month {
                p.text_faint
            } else if let Some(hue) = hue {
                p.hue(hue).1
            } else {
                p.text
            })
        }
    })
    .range(|prev| move |t: &Theme, s: &_| prev(t, s).background(t.ext::<Tokens>().hue(Hue::Blue).0))
    .events(|prev| move |t: &Theme, s: &_| prev(t, s).bottom(4))
    .event(|_| {
        |_: &Theme, s: &Look<kivact::ui::CalendarEventVisual>| {
            style()
                .size(4, 4)
                .radius(2)
                .background(if s.selected { Color::WHITE } else { s.color })
        }
    })
}
pub(crate) fn date_input(r: DateInputRecipes) -> DateInputRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(4).color(t.ext::<Tokens>().text))
}
pub(crate) fn date_range_input(r: DateRangeInputRecipes) -> DateRangeInputRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(8).color(t.ext::<Tokens>().text))
}
