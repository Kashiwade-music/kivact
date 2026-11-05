//! Role: Locale-backed month names and calendar caption controls.
use super::state::{CalendarCaption, Shared};
use kivact_core::i18n::{CalendarMonth, NameWidth, Weekday, message, use_locale};
use kivact_core::state::{Value, binding, memo};
use kivact_core::view::{IntoView, View, row, text};
use kivact_interaction::ListItem;
use std::rc::Rc;

pub(super) struct Names {
    pub(super) previous: Value<String>,
    pub(super) next: Value<String>,
}
impl Names {
    pub(super) fn new() -> Self {
        Self {
            previous: message("calendar.previous", "Previous month"),
            next: message("calendar.next", "Next month"),
        }
    }
    pub(super) fn weekday(&self, day: Weekday) -> Value<String> {
        let locale = use_locale();
        memo(move || locale.get().weekday_name(day, NameWidth::Short))
    }
}
pub(super) fn title(
    shared: &Rc<Shared>,
    _: &Rc<Names>,
    index: usize,
    month: CalendarMonth,
) -> View {
    let locale = use_locale();
    match shared.caption {
        CalendarCaption::Label => {
            let custom = shared.month_title.clone();
            text(move || {
                custom.as_ref().map_or_else(
                    || {
                        format!(
                            "{} {}",
                            locale.get().month_name(month.month(), NameWidth::Long),
                            month.year()
                        )
                    },
                    |title| title(month.first_day()),
                )
            })
            .into_view()
        }
        CalendarCaption::Dropdown => {
            let base = shared.month;
            let shift = index as i32;
            let months = move || {
                (1u8..=12)
                    .map(|month| {
                        ListItem::new(month, locale.get().month_name(month, NameWidth::Short))
                    })
                    .collect::<Vec<_>>()
            };
            let years = (month.year() - 50..=month.year() + 50)
                .map(|year| ListItem::new(year, year.to_string()))
                .collect::<Vec<_>>();
            let selected_month = binding(
                move || Some(base.get().add_months(shift).month()),
                move |next| {
                    if let Some(next) = next {
                        let at = base.get_untracked().add_months(shift);
                        base.set(CalendarMonth::new(at.year(), next).add_months(-shift));
                    }
                },
            );
            let selected_year = binding(
                move || Some(base.get().add_months(shift).year()),
                move |next| {
                    if let Some(next) = next {
                        let at = base.get_untracked().add_months(shift);
                        base.set(CalendarMonth::new(next, at.month()).add_months(-shift));
                    }
                },
            );
            row((
                crate::select(message("calendar.month", "Month"), months, selected_month),
                crate::select(message("calendar.year", "Year"), years, selected_year),
            ))
            .into_view()
        }
    }
}
