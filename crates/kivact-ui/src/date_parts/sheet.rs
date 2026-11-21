//! Role: Immediate date selection with a sheet-owned month/year panel.
use kivact_core::i18n::{CalendarDate, CalendarMonth, NameWidth, message, use_locale};
use kivact_core::state::{Signal, binding, signal};
use kivact_core::view::{Element, IntoView, Node, column, dynamic, row};
#[derive(Clone, Copy, PartialEq)]
enum DateSheetPanel {
    Calendar,
    MonthYear,
}
pub(crate) fn panel(value: Signal<Option<CalendarDate>>, picker: crate::Calendar) -> Node {
    let initial = value
        .get_untracked()
        .unwrap_or_else(CalendarDate::today)
        .month_of();
    let month = picker.sheet_month(initial);
    let locale = use_locale();
    let panel = signal(DateSheetPanel::Calendar);
    let title = crate::button(move || {
        format!(
            "{} {}",
            locale
                .get()
                .month_name(month.get().month(), NameWidth::Long),
            month.get().year()
        )
    })
    .on_press(move || panel.set(DateSheetPanel::MonthYear));
    let year = binding(
        move || f64::from(month.get().year()),
        move |year| {
            let current = month.get_untracked();
            month.set(CalendarMonth::new(year as i32, current.month()));
        },
    );
    let number = binding(
        move || f64::from(month.get().month()),
        move |number| {
            let current = month.get_untracked();
            month.set(CalendarMonth::new(current.year(), number as u8));
        },
    );
    let wheels = dynamic(
        move || panel.get() == DateSheetPanel::MonthYear,
        move |show| {
            if show {
                column((
                    row((
                        crate::date_parts::wheel::wheel(
                            message("date.month", "Month"),
                            number,
                            1.0,
                            12.0,
                            false,
                        ),
                        crate::date_parts::wheel::wheel(
                            message("date.year", "Year"),
                            year,
                            1.0,
                            9999.0,
                            false,
                        ),
                    )),
                    crate::button(message("date.done", "Done"))
                        .on_press(move || panel.set(DateSheetPanel::Calendar)),
                ))
                .into_view()
            } else {
                kivact_core::view::node().into_view()
            }
        },
    );
    column((
        title,
        wheels,
        picker
            .month(month)
            .touch()
            .hidden(move || panel.get() == DateSheetPanel::MonthYear),
    ))
}
