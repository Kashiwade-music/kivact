//! Role: Demonstrate the Soft calendar and controlled date and time fields.
use super::specimen::specimen;
use kivact::i18n::{CalendarDate, DateRange};
use kivact::state::signal;
use kivact::style::style;
use kivact::view::{Node, column};
use kivact_ui_soft::{
    DatePresentation, DateRangePreset, DateTime, TimeOfDay, calendar, date_input, date_range_input,
    date_time_input, time_input,
};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "calendar" => specimen(
            "Choose a date",
            calendar(signal(None::<CalendarDate>)),
            "calendar(date)",
        ),
        "date-input" => column((
            specimen(
                "Edit a date or open the calendar",
                date_input("Due date", signal(None::<CalendarDate>))
                    .presentation(DatePresentation::Desktop),
                "date_input(\"Due date\", date).presentation(DatePresentation::Desktop)",
            ),
            specimen(
                "Calendar and month or year wheels",
                date_input(
                    "Touch due date",
                    signal(Some(CalendarDate::new(2026, 10, 14))),
                )
                .presentation(DatePresentation::BottomSheet),
                "date_input(\"Touch due date\", date).presentation(DatePresentation::BottomSheet)",
            ),
        ))
        .style(style().gap(12)),
        "date-range-input" => {
            let date = |day| CalendarDate::new(2026, 10, day);
            let week = DateRange::new(date(12), date(18));
            specimen(
                "Choose a start and end or pick a preset",
                date_range_input("Booking dates", signal(None::<DateRange>))
                    .presets(vec![DateRangePreset::new("Example week", move || week)]),
                "date_range_input(\"Booking dates\", dates).presets(presets)",
            )
        }
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
