//! Role: Format timestamps as calendar dates, wall-clock times, or relative ages.
use crate::{TextColor, TextKind, TextSize, TextWeight, styled_text};
use kivact::Prop;
use kivact::component;
use kivact::i18n::{
    CalendarDate, Locale, NameWidth, TimeOfDay, TimeStyle, Weekday, message, use_locale,
};
use kivact::state::Value;
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
    let locale = use_locale();
    // The instant the component mounts, used while the text is not live.
    let mounted = TimestampValue::from(SystemTime::now());
    let ticks = kivact::task::clock(Duration::from_secs(1));
    let relative = memo(move || {
        matches!(
            format.get(),
            TimestampFormat::Relative | TimestampFormat::RelativeShort | TimestampFormat::Auto
        )
    });
    let ticking = memo(move || live.get() && relative.get());
    let now = memo(move || {
        if ticking.get() {
            TimestampValue::from(ticks.get())
        } else {
            mounted
        }
    });
    let now_label = message("soft.relative_time.now", "now");
    let yesterday = message("soft.relative_time.yesterday", "yesterday");
    let display = memo(move || {
        let instant = value.get();
        let now = now.get();
        let format = resolved_format(format.get(), instant, now, auto_threshold.get());
        timestamp_text(
            instant,
            format,
            &time_zone.get(),
            show_time_zone.get(),
            now,
            (now_label, yesterday),
            &locale.get(),
        )
    });
    let full = move || {
        calendar_text(
            value.get(),
            TimestampFormat::DateTime,
            &time_zone.get(),
            true,
            true,
            &locale.get(),
        )
    };
    let name = move || {
        if relative.get() {
            full()
        } else {
            display.get()
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
        move || timestamp_details(value, time_zone, entries, locale),
        trigger,
    )
    .enabled(enabled)
}

/// The format to show: `Auto` picks a relative age inside the threshold and a date-time beyond it.
fn resolved_format(
    format: TimestampFormat,
    instant: TimestampValue,
    now: TimestampValue,
    threshold: Duration,
) -> TimestampFormat {
    if format != TimestampFormat::Auto {
        format
    } else if difference(now, instant).unsigned_abs() <= threshold.as_secs() {
        TimestampFormat::Relative
    } else {
        TimestampFormat::DateTime
    }
}

/// The text of a resolved format. `words` are the relative labels for now and yesterday.
fn timestamp_text(
    instant: TimestampValue,
    format: TimestampFormat,
    zone: &TimeZone,
    show_zone: bool,
    now: TimestampValue,
    words: (Value<String>, Value<String>),
    locale: &Locale,
) -> String {
    match format {
        TimestampFormat::Relative | TimestampFormat::RelativeShort => relative_text(
            difference(now, instant),
            format == TimestampFormat::RelativeShort,
            words.0,
            words.1,
        ),
        _ => calendar_text(instant, format, zone, show_zone, false, locale),
    }
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

/// The weekday of a day count from the Unix epoch, which was a Thursday.
fn weekday_of(days: i64) -> Weekday {
    match (days + 3).rem_euclid(7) {
        0 => Weekday::Monday,
        1 => Weekday::Tuesday,
        2 => Weekday::Wednesday,
        3 => Weekday::Thursday,
        4 => Weekday::Friday,
        5 => Weekday::Saturday,
        _ => Weekday::Sunday,
    }
}

fn calendar_text(
    instant: TimestampValue,
    format: TimestampFormat,
    zone: &TimeZone,
    show_zone: bool,
    full: bool,
    locale: &Locale,
) -> String {
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
    let time = TimeOfDay {
        hour: hour as u8,
        minute: minute as u8,
        second: second as u8,
    };
    let month_name = locale.month_name(date.month(), NameWidth::Long);
    let short = locale.month_name(date.month(), NameWidth::Short);
    let mut clock = locale.format_time(time, TimeStyle { seconds: full });
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
            locale.weekday_name(weekday_of(days), NameWidth::Short)
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

fn timestamp_details(
    value: Value<TimestampValue>,
    zone: Value<TimeZone>,
    entries: Value<Vec<TimestampEntry>>,
    locale: Value<Locale>,
) -> String {
    let entries = entries.get();
    let locale = locale.get();
    if entries.is_empty() {
        return calendar_text(
            value.get(),
            TimestampFormat::DateTime,
            &zone.get(),
            true,
            true,
            &locale,
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
                &locale,
            );
            entry
                .label
                .as_ref()
                .map_or_else(|| value.clone(), |label| format!("{label}  {value}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
