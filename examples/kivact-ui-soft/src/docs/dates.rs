//! Role: Demonstrate the Soft calendar and controlled date and time fields.
use super::specimen::specimen;
use kivact::i18n::{CalendarDate, DateRange};
use kivact::state::signal;
use kivact::style::style;
use kivact::view::{Node, column};

pub(super) fn examples(slug: &str) -> Option<Node> {
    Some(match slug {
        "calendar" => specimen(
            "Choose a date",
            kivact::ui::calendar(signal(None::<CalendarDate>)),
            "kivact::ui::calendar(date)",
        ),
        "date-input" => column((
            specimen(
                "Edit a date or open the calendar",
                kivact::ui::date_input("Due date", signal(None::<CalendarDate>))
                    .presentation(kivact::ui::DatePresentation::Desktop),
                "kivact::ui::date_input(\"Due date\", date).presentation(kivact::ui::DatePresentation::Desktop)",
            ),
            specimen(
                "Calendar and month or year wheels",
                kivact::ui::date_input(
                    "Touch due date",
                    signal(Some(CalendarDate::new(2026, 10, 14))),
                )
                .presentation(kivact::ui::DatePresentation::BottomSheet),
                "kivact::ui::date_input(\"Touch due date\", date).presentation(kivact::ui::DatePresentation::BottomSheet)",
            ),
        ))
        .style(style().gap(12)),
        "date-range-input" => {
            let date = |day| CalendarDate::new(2026, 10, day);
            let week = DateRange::new(date(12), date(18));
            specimen(
                "Choose a start and end or pick a preset",
                kivact::ui::date_range_input("Booking dates", signal(None::<DateRange>))
                    .presets(vec![kivact::ui::DateRangePreset::new("Example week", move || week)]),
                "kivact::ui::date_range_input(\"Booking dates\", dates).presets(presets)",
            )
        }
        "date-time-input" => specimen(
            "Edit date and time",
            kivact::ui::date_time_input("Starts at", signal(None::<kivact::i18n::DateTime>)),
            "kivact::ui::date_time_input(\"Starts at\", starts_at)",
        ),
        "time-input" => specimen(
            "Edit time",
            kivact::ui::time_input("Reminder time", signal(None::<kivact::i18n::TimeOfDay>)),
            "kivact::ui::time_input(\"Reminder time\", time)",
        ),
        _ => return None,
    })
}
