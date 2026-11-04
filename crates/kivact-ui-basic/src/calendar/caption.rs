//! Role: Calendar texts and the month title: a label or month and year selects.

use super::state::{CalendarCaption, Shared};
use crate::Color;
use crate::list_popup::select_popup;
use crate::theme_access::{focus_ring, tokens};
use kivact::i18n::{CalendarMonth, Weekday, message, message_with};
use kivact::interaction::ListItem;
use kivact::interaction::use_select;
use kivact::prelude::NodeBuilder;
use kivact::state::IntoValue;
use kivact::state::{Binding, Value, binding};
use kivact::style::{Align, Direction, Justify, style};
use kivact::view::icon;
use kivact::view::{IntoView, View};
use kivact::view::{node, row, text};
use std::rc::Rc;

const MONTHS: [(&str, &str); 12] = [
    ("basic.calendar.month_1", "January"),
    ("basic.calendar.month_2", "February"),
    ("basic.calendar.month_3", "March"),
    ("basic.calendar.month_4", "April"),
    ("basic.calendar.month_5", "May"),
    ("basic.calendar.month_6", "June"),
    ("basic.calendar.month_7", "July"),
    ("basic.calendar.month_8", "August"),
    ("basic.calendar.month_9", "September"),
    ("basic.calendar.month_10", "October"),
    ("basic.calendar.month_11", "November"),
    ("basic.calendar.month_12", "December"),
];

const SHORT: [(&str, &str); 12] = [
    ("basic.calendar.short_1", "Jan"),
    ("basic.calendar.short_2", "Feb"),
    ("basic.calendar.short_3", "Mar"),
    ("basic.calendar.short_4", "Apr"),
    ("basic.calendar.short_5", "May"),
    ("basic.calendar.short_6", "Jun"),
    ("basic.calendar.short_7", "Jul"),
    ("basic.calendar.short_8", "Aug"),
    ("basic.calendar.short_9", "Sep"),
    ("basic.calendar.short_10", "Oct"),
    ("basic.calendar.short_11", "Nov"),
    ("basic.calendar.short_12", "Dec"),
];

/// The messages a calendar shows, read once at setup.
pub(crate) struct Names {
    weekdays: [Value<String>; 7],
    months: [Value<String>; 12],
    short: [Value<String>; 12],
    pub(super) previous: Value<String>,
    pub(super) next: Value<String>,
    month_label: Value<String>,
    year_label: Value<String>,
}

impl Names {
    pub(crate) fn new() -> Self {
        Names {
            weekdays: [
                message("basic.calendar.mon", "Mon"),
                message("basic.calendar.tue", "Tue"),
                message("basic.calendar.wed", "Wed"),
                message("basic.calendar.thu", "Thu"),
                message("basic.calendar.fri", "Fri"),
                message("basic.calendar.sat", "Sat"),
                message("basic.calendar.sun", "Sun"),
            ],
            months: MONTHS.map(|(key, default)| message(key, default)),
            short: SHORT.map(|(key, default)| message(key, default)),
            previous: message("basic.calendar.previous", "Previous month"),
            next: message("basic.calendar.next", "Next month"),
            month_label: message("basic.calendar.month", "Month"),
            year_label: message("basic.calendar.year", "Year"),
        }
    }

    /// The name of a month, 1 to 12.
    pub(crate) fn month(&self, month: u8) -> Value<String> {
        self.months[usize::from(month.clamp(1, 12)) - 1]
    }

    /// The short name of a weekday.
    pub(super) fn weekday(&self, day: Weekday) -> Value<String> {
        self.weekdays[day.days_since(Weekday::Monday)]
    }
}

/// A select trigger that reads like the caption: `text-sm font-medium` and a
/// small chevron.
fn dropdown<K: kivact::state::ItemKey>(
    name: Value<String>,
    options: Vec<ListItem<K>>,
    value: Binding<Option<K>>,
) -> View {
    let options = options.into_value();
    let behavior = use_select(options, value);
    let trigger = behavior.trigger();
    let anchor = kivact::view::node_ref();
    let label = behavior.label();
    (
        node()
            .with_node(|root| root.part(trigger))
            .node_ref(anchor)
            .name(name)
            .child(text(label))
            .child(
                node()
                    .child(icon("chevron-down"))
                    .style(style().size(14, 14).color(Color::MutedForeground)),
            )
            .style(move || {
                let p = tokens();
                style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(4)
                    .height(28)
                    .padding_x(8)
                    .radius(0)
                    .border(1, kivact::style::Color::TRANSPARENT)
                    .font_size(14)
                    .weight(500)
                    .text_wrap(false)
                    .color(p.foreground)
                    .hover(style().background(p.muted))
                    .focus_visible(focus_ring(&p))
            }),
        select_popup(behavior, options, anchor, name),
    )
        .into_view()
}

/// The title of the month panel `index`, which shows `month`.
pub(super) fn title(
    shared: &Rc<Shared>,
    names: &Rc<Names>,
    index: usize,
    month: CalendarMonth,
) -> View {
    match shared.caption {
        CalendarCaption::Label => {
            let name = names.months[usize::from(month.month()) - 1];
            let year = month.year();
            let label = message_with("basic.calendar.caption", "{month} {year}", move || {
                [("month", name.get()), ("year", year.to_string())]
            });
            text(label)
                .style(style().font_size(14).weight(500).text_wrap(false))
                .into_view()
        }
        CalendarCaption::Dropdown => {
            let base = shared.month;
            let shift = index as i32;
            let months = (1u8..=12)
                .map(|m| ListItem::new(m, names.short[usize::from(m) - 1]))
                .collect::<Vec<_>>();
            let years = (month.year() - 50..=month.year() + 50)
                .map(|y| ListItem::new(y, y.to_string()))
                .collect::<Vec<_>>();
            let month_value = binding(
                move || Some(base.get().add_months(shift).month()),
                move |chosen: Option<u8>| {
                    if let Some(m) = chosen {
                        let at = base.get().add_months(shift);
                        base.set(CalendarMonth::new(at.year(), m).add_months(-shift));
                    }
                },
            );
            let year_value = binding(
                move || Some(base.get().add_months(shift).year()),
                move |chosen: Option<i32>| {
                    if let Some(y) = chosen {
                        let at = base.get().add_months(shift);
                        base.set(CalendarMonth::new(y, at.month()).add_months(-shift));
                    }
                },
            );
            row((
                dropdown(names.month_label, months, month_value),
                dropdown(names.year_label, years, year_value),
            ))
            .style(
                style()
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .gap(6),
            )
            .into_view()
        }
    }
}
