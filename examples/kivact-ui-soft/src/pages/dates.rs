//! Demonstrate the Soft calendar and controlled date and time fields.
use super::labelled;
use kivact::i18n::{CalendarDate, DateRange};
use kivact::state::signal;
use kivact::style::style;
use kivact::view::{Node, column};

// example: calendar
pub(super) fn example_calendar() -> Node {
    labelled(
        "Choose a date",
        kivact::ui::calendar(signal(None::<CalendarDate>)),
    )
}
// end example

// example: date-input
pub(super) fn example_date_input() -> Node {
    column((
        labelled(
            "Edit a date or open the calendar",
            kivact::ui::date_input(signal(None::<CalendarDate>))
                .label("Due date")
                .presentation(kivact::ui::DatePresentation::Desktop),
        ),
        labelled(
            "Calendar and month or year wheels",
            kivact::ui::date_input(signal(Some(CalendarDate::new(2026, 10, 14))))
                .label("Touch due date")
                .presentation(kivact::ui::DatePresentation::BottomSheet),
        ),
    ))
    .style(style().gap(12))
}
// end example

// example: date-range-input
pub(super) fn example_date_range_input() -> Node {
    {
        let date = |day| CalendarDate::new(2026, 10, day);
        let week = DateRange::new(date(12), date(18));
        labelled(
            "Choose a start and end or pick a preset",
            kivact::ui::date_range_input(signal(None::<DateRange>))
                .label("Booking dates")
                .presets(vec![kivact::ui::DateRangePreset::new(
                    "Example week",
                    move || week,
                )]),
        )
    }
}
// end example

// example: date-time-input
pub(super) fn example_date_time_input() -> Node {
    labelled(
        "Edit date and time",
        kivact::ui::date_time_input(signal(None::<kivact::i18n::DateTime>)).label("Starts at"),
    )
}
// end example

// example: time-input
pub(super) fn example_time_input() -> Node {
    labelled(
        "Edit time",
        kivact::ui::time_input(signal(None::<kivact::i18n::TimeOfDay>)).label("Reminder time"),
    )
}
// end example
