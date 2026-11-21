//! Role: Project a date-time value into its date and time fields.
use kivact_core::i18n::{CalendarDate, DateTime, TimeOfDay};
use kivact_core::state::{Signal, Value, binding};
fn key(value: DateTime) -> (CalendarDate, u8, u8, u8) {
    (
        value.date,
        value.time.hour,
        value.time.minute,
        value.time.second,
    )
}
/// Endpoint bindings project the caller-owned accepted date/time value.
pub(crate) struct DateTimeFields {
    pub(crate) date: Signal<Option<CalendarDate>>,
    pub(crate) time: Signal<Option<TimeOfDay>>,
}
/// Preserve the complementary endpoint while validating each accepted write.
pub(crate) fn field_bindings(
    value: Signal<Option<DateTime>>,
    min: Value<Option<DateTime>>,
    max: Value<Option<DateTime>>,
) -> DateTimeFields {
    let accepted = binding(value, move |next: Option<DateTime>| {
        if next.is_none_or(|next| {
            min.get().is_none_or(|min| key(next) >= key(min))
                && max.get().is_none_or(|max| key(next) <= key(max))
        }) {
            value.set(next);
        }
    });
    let date = binding(
        move || value.get().map(|value| value.date),
        move |next| {
            accepted.set(next.map(|date| DateTime {
                date,
                time: value.get_untracked().map_or(
                    TimeOfDay {
                        hour: 0,
                        minute: 0,
                        second: 0,
                    },
                    |value| value.time,
                ),
            }))
        },
    );
    let time = binding(
        move || value.get().map(|value| value.time),
        move |next| {
            accepted.set(next.map(|time| {
                DateTime {
                    time,
                    date: value
                        .get_untracked()
                        .map_or_else(CalendarDate::today, |value| value.date),
                }
            }))
        },
    );
    DateTimeFields { date, time }
}
