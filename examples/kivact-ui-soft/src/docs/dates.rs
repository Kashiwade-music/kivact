//! Role: Demonstrate the Soft calendar and controlled date and time fields.
use super::specimen::specimen;
use kivact::control::Node;
use kivact::i18n::{CalendarDate, DateRange};
use kivact::signal;
use kivact_ui_soft::{
    DateTime, TimeOfDay, calendar, date_input, date_range_input, date_time_input, time_input,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "calendar" => specimen(
            "Choose a date",
            calendar(signal(None::<CalendarDate>)),
            "calendar(date)",
        ),
        "date-input" => specimen(
            "Edit a date or open the calendar",
            date_input("Due date", signal(None::<CalendarDate>)),
            "date_input(\"Due date\", date)",
        ),
        "date-range-input" => specimen(
            "Choose a start and end",
            date_range_input("Booking dates", signal(None::<DateRange>)),
            "date_range_input(\"Booking dates\", dates)",
        ),
        "date-time-input" => specimen(
            "Edit date and time",
            date_time_input("Starts at", signal(None::<DateTime>)),
            "date_time_input(\"Starts at\", starts_at)",
        ),
        "time-input" => specimen(
            "Edit time",
            time_input("Reminder time", signal(None::<TimeOfDay>)),
            "time_input(\"Reminder time\", time)",
        ),
        _ => return None,
    })
}
