//! Role: Format timestamps as calendar dates, wall-clock times, or relative ages.
use crate::{TextColor, TextKind, TextSize, TextWeight, styled_text};
use kivact::Prop;
use kivact::component;
use kivact::i18n::CalendarDate;
use kivact::i18n::message;
use kivact::state::Machine;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::machine;
use kivact::state::memo;
use kivact::ui::tooltip;
use kivact::view::node;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A UTC instant measured in Unix milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Prop)]
pub struct TimestampValue(i64);

impl TimestampValue {
    /// Create an instant from Unix seconds.
    pub fn unix_seconds(seconds: i64) -> Self {
        Self(seconds.saturating_mul(1000))
    }
    /// Create an instant from Unix milliseconds.
    pub fn unix_millis(milliseconds: i64) -> Self {
        Self(milliseconds)
    }
    /// Interpret finite values below 10¹² as seconds and other values as milliseconds.
    pub fn from_number(number: f64) -> Option<Self> {
        let milliseconds = if number < 1e12 {
            number * 1000.0
        } else {
            number
        };
        (milliseconds.is_finite()
            && milliseconds >= i64::MIN as f64
            && milliseconds < -(i64::MIN as f64))
            .then(|| Self(milliseconds.trunc() as i64))
    }

    /// Parse an ISO Gregorian date or date-time with an optional numeric offset.
    /// A date-time without an offset is interpreted as UTC.
    pub fn parse_iso(input: &str) -> Option<Self> {
        let (date, clock) = input
            .split_once(['T', ' '])
            .map_or((input, None), |(date, clock)| (date, Some(clock)));
        let mut fields = date.split('-');
        let year: i64 = fields.next()?.parse().ok()?;
        let month: i64 = fields.next()?.parse().ok()?;
        let day: i64 = fields.next()?.parse().ok()?;
        if fields.next().is_some()
            || !(0..=9999).contains(&year)
            || !(1..=12).contains(&month)
            || !(1..=31).contains(&day)
        {
            return None;
        }
        let date = CalendarDate::checked(
            i32::try_from(year).ok()?,
            u8::try_from(month).ok()?,
            u8::try_from(day).ok()?,
        )?;
        let days = date.to_days();
        let Some(clock) = clock else {
            return Some(Self::unix_seconds(days * 86_400));
        };
        let (clock, offset) =
            if let Some(clock) = clock.strip_suffix('Z').or_else(|| clock.strip_suffix('z')) {
                (clock, 0)
            } else if let Some(index) = clock.find(['+', '-']) {
                let (clock, offset) = clock.split_at(index);
                let (sign, offset) = offset.split_at(1);
                let (hours, minutes) = match offset.split_once(':') {
                    Some(parts) => parts,
                    None if offset.len() == 4 => (offset.get(..2)?, offset.get(2..)?),
                    None => return None,
                };
                let hours: i64 = hours.parse().ok()?;
                let minutes: i64 = minutes.parse().ok()?;
                if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
                    return None;
                }
                (
                    clock,
                    (hours * 3600 + minutes * 60) * if sign == "-" { -1 } else { 1 },
                )
            } else {
                (clock, 0)
            };
        let mut fields = clock.split(':');
        let hours: i64 = fields.next()?.parse().ok()?;
        let minutes: i64 = fields.next()?.parse().ok()?;
        let seconds = fields.next().unwrap_or("0");
        let (seconds, fraction) = seconds.split_once('.').unwrap_or((seconds, ""));
        let seconds: i64 = seconds.parse().ok()?;
        if fields.next().is_some()
            || !(0..=23).contains(&hours)
            || !(0..=59).contains(&minutes)
            || !(0..=59).contains(&seconds)
            || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        let milliseconds = fraction
            .bytes()
            .take(3)
            .zip([100, 10, 1])
            .map(|(byte, scale)| i64::from(byte - b'0') * scale)
            .sum::<i64>();
        Some(Self(
            (days * 86_400 + hours * 3600 + minutes * 60 + seconds - offset) * 1000 + milliseconds,
        ))
    }
}

impl From<SystemTime> for TimestampValue {
    fn from(time: SystemTime) -> Self {
        let milliseconds = match time.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis() as i128,
            Err(error) => -(error.duration().as_millis() as i128),
        };
        Self(milliseconds.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64)
    }
}

/// The application-supplied offset and display name for a time zone.
#[derive(Clone, Debug, Default, PartialEq, Eq, Prop)]
pub struct TimeZone {
    /// The local offset from UTC in seconds for the instant being rendered.
    pub offset_seconds: i32,
    /// An optional short name such as UTC or PDT.
    pub label: Option<String>,
}

/// The format of a detail entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InstantFormat {
    #[default]
    /// Show a long date and clock including seconds.
    Full,
    /// Show the date with an abbreviated month name.
    Date,
    /// Show the date and a twelve-hour clock.
    DateTime,
    /// Show a twelve-hour clock.
    Time,
    /// Show the numeric date as year-month-day.
    SystemDate,
    /// Show the numeric date and a twenty-four-hour clock including seconds.
    SystemDateTime,
    /// Show a twenty-four-hour clock including seconds.
    SystemTime,
    /// Show the Unix timestamp in whole seconds.
    UnixSeconds,
}

/// A row in the timestamp details.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimestampEntry {
    /// Optional time zone overriding the main timestamp zone for this entry.
    pub zone: Option<TimeZone>,
    /// Date and time format for this detail row.
    pub format: InstantFormat,
    /// Optional label preceding the detail value.
    pub label: Option<String>,
    /// Reserved for a copy action on this detail row; the timestamp details do not draw one yet, so the value has no effect.
    pub copyable: bool,
}

/// The format of the main timestamp.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum TimestampFormat {
    /// Show a relative age in words.
    Relative,
    /// Show a compact relative age.
    RelativeShort,
    #[default]
    /// Use a relative age near the current time and a date-time beyond the threshold.
    Auto,
    /// Show the date with an abbreviated month name.
    Date,
    /// Show the date with the full month name.
    DateLong,
    /// Show the weekday and date.
    DateWeekday,
    /// Show the date and a twelve-hour clock.
    DateTime,
    /// Show a twelve-hour clock.
    Time,
    /// Show the numeric date as year-month-day.
    SystemDate,
    /// Show the numeric date and a twenty-four-hour clock including seconds.
    SystemDateTime,
    /// Show a twenty-four-hour clock including seconds.
    SystemTime,
    /// Show the Unix timestamp in whole seconds.
    UnixSeconds,
}

/// The settings the presentation reads, gathered from the component's props.
#[derive(Clone, Copy)]
struct TimestampSettings {
    value: Value<TimestampValue>,
    format: Value<TimestampFormat>,
    threshold: Value<Duration>,
    zone: Value<TimeZone>,
    show_zone: Value<bool>,
}

/// A date or relative timestamp with optional details.
///
/// `format` selects the presentation and `auto_threshold` the window of `Auto`. `hover_card`
/// enables the details tooltip, which shows `entries` or the full date. `live` updates the
/// relative text while mounted. The text is wrapped in a node that carries the caller's
/// modifiers and the accessible name; `kind`, `color`, `size` and `weight` set its typography.
#[component]
pub fn timestamp(
    value: Value<TimestampValue>,
    #[prop(default = TimestampFormat::Auto)] format: Value<TimestampFormat>,
    #[prop(default = Duration::from_secs(604_800))] auto_threshold: Value<Duration>,
    #[prop(default = true)] hover_card: Value<bool>,
    #[prop(default = Vec::new())] entries: Value<Vec<TimestampEntry>>,
    #[prop(default = TimeZone::default())] time_zone: Value<TimeZone>,
    #[prop(default = false)] show_time_zone: Value<bool>,
    #[prop(default = false)] live: Value<bool>,
    #[prop(default = TextKind::Supporting)] kind: Value<TextKind>,
    #[prop(default = TextColor::Secondary)] color: Value<TextColor>,
    size: Option<Value<TextSize>>,
    weight: Option<Value<TextWeight>>,
) -> impl Element {
    let settings = TimestampSettings {
        value,
        format,
        threshold: auto_threshold,
        zone: time_zone,
        show_zone: show_time_zone,
    };
    let clock = machine(
        TimestampClock::Still {
            now: TimestampValue::from(SystemTime::now()),
        },
        move || TimestampTick {
            value: value.get(),
            now: TimestampValue::from(SystemTime::now()),
        },
        |_, ()| {},
    );
    let now_label = message("soft.relative_time.now", "now");
    let yesterday = message("soft.relative_time.yesterday", "yesterday");
    let display = memo(move || {
        let now = clock.get().now();
        timestamp_presentation(&settings, now, now_label, yesterday)
    });
    let relative = memo(move || {
        matches!(
            format.get(),
            TimestampFormat::Relative | TimestampFormat::RelativeShort | TimestampFormat::Auto
        )
    });
    kivact::state::effect(move || {
        if live.get() && relative.get() {
            clock.send(TimestampEvent::Live);
        } else {
            clock.send(TimestampEvent::Still);
        }
    });
    let full = move || {
        calendar_text(
            value.get(),
            TimestampFormat::DateTime,
            &time_zone.get(),
            true,
            true,
        )
    };
    let name = move || {
        if relative.get() {
            full()
        } else {
            let now = clock.get().now();
            timestamp_presentation(&settings, now, now_label, yesterday)
        }
    };
    let enabled = memo(move || {
        hover_card.get() && (relative.get() || entries.with(|entries| !entries.is_empty()))
    });
    let mut text = styled_text(display).kind(kind).color(color);
    if let Some(size) = size {
        text = text.size(size);
    }
    if let Some(weight) = weight {
        text = text.weight(weight);
    }
    // Standard tooltip fallback until Soft's interactive hover card is integrated.
    let trigger = node().child(text).name(name).focusable(enabled);
    tooltip(
        move || timestamp_details(value, time_zone, entries),
        trigger,
    )
    .enabled(enabled)
}

fn difference(now: TimestampValue, then: TimestampValue) -> i64 {
    now.0
        .saturating_sub(then.0)
        .saturating_add(500)
        .div_euclid(1000)
}

fn relative_text(diff: i64, short: bool, now: Value<String>, yesterday: Value<String>) -> String {
    let age = diff.unsigned_abs();
    if age < 10 || (diff < 0 && age <= 30) {
        return now.get();
    }
    let (divisor, unit, abbreviation) = match age {
        0..=59 => (1, "second", "s"),
        60..=3599 => (60, "minute", "m"),
        3600..=86399 => (3600, "hour", "h"),
        86400..=2591999 => (86400, "day", "d"),
        2592000..=31535999 => (2592000, "month", "mo"),
        _ => (31536000, "year", "y"),
    };
    let count = age / divisor;
    if !short && diff > 0 && divisor == 86400 && count == 1 {
        return yesterday.get();
    }
    let amount = if short {
        format!("{count}{abbreviation}")
    } else {
        format!("{count} {unit}{}", if count == 1 { "" } else { "s" })
    };
    if diff > 0 {
        format!("{amount} ago")
    } else {
        format!("in {amount}")
    }
}

fn calendar_text(
    instant: TimestampValue,
    format: TimestampFormat,
    zone: &TimeZone,
    show_zone: bool,
    full: bool,
) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    const WEEKDAYS: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    let unix = instant.0.div_euclid(1000);
    let local = unix + i64::from(zone.offset_seconds);
    let days = local.div_euclid(86_400);
    let date = CalendarDate::from_days(days);
    let (year, month, day) = (
        i64::from(date.year()),
        i64::from(date.month()),
        i64::from(date.day()),
    );
    let hour = local.div_euclid(3600).rem_euclid(24);
    let minute = local.div_euclid(60).rem_euclid(60);
    let second = local.rem_euclid(60);
    let month_name = MONTHS[(month - 1) as usize];
    let short = &month_name[..3];
    let hour12 = (hour + 11) % 12 + 1;
    let ampm = if hour < 12 { "AM" } else { "PM" };
    let mut clock = if full {
        format!("{hour12}:{minute:02}:{second:02} {ampm}")
    } else {
        format!("{hour12}:{minute:02} {ampm}")
    };
    if show_zone && let Some(name) = zone.label.as_deref() {
        clock.push(' ');
        clock.push_str(name);
    }
    if full {
        return format!("{month_name} {day}, {year} at {clock}");
    }
    match format {
        TimestampFormat::Date => format!("{short} {day}, {year}"),
        TimestampFormat::DateLong => format!("{month_name} {day}, {year}"),
        TimestampFormat::DateWeekday => format!(
            "{}, {short} {day}, {year}",
            WEEKDAYS[days.rem_euclid(7) as usize]
        ),
        TimestampFormat::Time => clock,
        TimestampFormat::SystemDate => format!("{year:04}-{month:02}-{day:02}"),
        TimestampFormat::SystemDateTime => {
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
        }
        TimestampFormat::SystemTime => format!("{hour:02}:{minute:02}:{second:02}"),
        TimestampFormat::UnixSeconds => unix.to_string(),
        _ => format!("{short} {day}, {year}, {clock}"),
    }
}

#[derive(Clone, PartialEq)]
enum TimestampClock {
    /// Refreshed once at `now`; no tick is scheduled.
    Still { now: TimestampValue },
    /// Refreshed at `now`; the next tick is scheduled.
    Live { now: TimestampValue },
}
impl TimestampClock {
    fn now(&self) -> TimestampValue {
        match self {
            Self::Still { now } | Self::Live { now } => *now,
        }
    }
}
struct TimestampTick {
    value: TimestampValue,
    now: TimestampValue,
}
impl Machine for TimestampTick {
    type State = TimestampClock;
    type Event = TimestampEvent;
    type Effect = ();
    fn transition(
        &self,
        state: &TimestampClock,
        event: TimestampEvent,
    ) -> Step<TimestampClock, TimestampEvent, ()> {
        let age = difference(self.now, self.value).unsigned_abs();
        let delay = match age {
            0..=59 => 1,
            60..=3599 => 30,
            3600..=86399 => 60,
            _ => 300,
        };
        match (state, event) {
            (_, TimestampEvent::Still) => {
                Step::to(TimestampClock::Still { now: self.now }).cancel(TimestampEvent::Tick)
            }
            (_, TimestampEvent::Live) | (TimestampClock::Live { .. }, TimestampEvent::Tick) => {
                Step::to(TimestampClock::Live { now: self.now })
                    .after(Duration::from_secs(delay), TimestampEvent::Tick)
            }
            (TimestampClock::Still { .. }, TimestampEvent::Tick) => Step::stay(),
        }
    }
}

enum TimestampEvent {
    /// Refresh once and stop ticking.
    Still,
    /// Refresh now and keep ticking.
    Live,
    /// The scheduled tick fired.
    Tick,
}
fn timestamp_presentation(
    settings: &TimestampSettings,
    now: TimestampValue,
    now_label: Value<String>,
    yesterday: Value<String>,
) -> String {
    let instant = settings.value.get();
    let diff = difference(now, instant);
    let format = if settings.format.get() == TimestampFormat::Auto {
        if diff.unsigned_abs() <= settings.threshold.get().as_secs() {
            TimestampFormat::Relative
        } else {
            TimestampFormat::DateTime
        }
    } else {
        settings.format.get()
    };
    match format {
        TimestampFormat::Relative | TimestampFormat::RelativeShort => relative_text(
            diff,
            format == TimestampFormat::RelativeShort,
            now_label,
            yesterday,
        ),
        _ => calendar_text(
            instant,
            format,
            &settings.zone.get(),
            settings.show_zone.get(),
            false,
        ),
    }
}

fn timestamp_details(
    value: Value<TimestampValue>,
    zone: Value<TimeZone>,
    entries: Value<Vec<TimestampEntry>>,
) -> String {
    let entries = entries.get();
    if entries.is_empty() {
        return calendar_text(
            value.get(),
            TimestampFormat::DateTime,
            &zone.get(),
            true,
            true,
        );
    }
    entries
        .iter()
        .map(|entry| {
            let (format, full) = match entry.format {
                InstantFormat::Full => (TimestampFormat::DateTime, true),
                InstantFormat::Date => (TimestampFormat::Date, false),
                InstantFormat::DateTime => (TimestampFormat::DateTime, false),
                InstantFormat::Time => (TimestampFormat::Time, false),
                InstantFormat::SystemDate => (TimestampFormat::SystemDate, false),
                InstantFormat::SystemDateTime => (TimestampFormat::SystemDateTime, false),
                InstantFormat::SystemTime => (TimestampFormat::SystemTime, false),
                InstantFormat::UnixSeconds => (TimestampFormat::UnixSeconds, false),
            };
            let value = calendar_text(
                value.get(),
                format,
                entry.zone.as_ref().unwrap_or(&zone.get()),
                entry.zone.is_some() || full,
                full,
            );
            entry
                .label
                .as_ref()
                .map_or_else(|| value.clone(), |label| format!("{label}  {value}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
