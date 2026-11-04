//! Role: Format timestamps as calendar dates, wall-clock times, or relative ages.
use crate::{Text, TextColor, TextKind, TextSize, TextWeight, text};
use kivact::control::{Node, NodeBuilder};
use kivact::i18n::message;
use kivact::state::{Machine, MachineHandle, Step, machine};
use kivact::ui::tooltip;
use kivact::view::{IntoValue, IntoView, View};
use kivact::{Value, memo};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A UTC instant measured in Unix milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(i64);

impl Instant {
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
        let days = days_from_civil(year, month, day);
        if civil_from_days(days) != (year, month, day) {
            return None;
        }
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

impl From<SystemTime> for Instant {
    fn from(time: SystemTime) -> Self {
        let milliseconds = match time.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis() as i128,
            Err(error) => -(error.duration().as_millis() as i128),
        };
        Self(milliseconds.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64)
    }
}

/// The application-supplied offset and display name for a time zone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    Full,
    Date,
    DateTime,
    Time,
    SystemDate,
    SystemDateTime,
    SystemTime,
    UnixSeconds,
}

/// A row in the timestamp details.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimestampEntry {
    pub zone: Option<TimeZone>,
    pub format: InstantFormat,
    pub label: Option<String>,
    pub copyable: bool,
}

/// The format of the main timestamp.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimestampFormat {
    Relative,
    RelativeShort,
    #[default]
    Auto,
    Date,
    DateLong,
    DateWeekday,
    DateTime,
    Time,
    SystemDate,
    SystemDateTime,
    SystemTime,
    UnixSeconds,
}

/// A date or relative timestamp with optional details.
pub struct Timestamp {
    text: Text,
    clock: MachineHandle<TimestampTick>,
    settings: Rc<RefCell<TimestampSettings>>,
    details: bool,
    entries: Vec<TimestampEntry>,
    live: bool,
}

/// Show a timestamp in automatic relative or absolute format.
pub fn timestamp(value: impl IntoValue<Instant>) -> Timestamp {
    let settings = Rc::new(RefCell::new(TimestampSettings {
        value: value.into_value(),
        format: TimestampFormat::Auto,
        threshold: Duration::from_secs(604_800),
        zone: TimeZone::default(),
        show_zone: false,
    }));
    let input = settings.clone();
    let clock = machine(
        TimestampClock::Running {
            now: Instant::from(SystemTime::now()),
        },
        move || TimestampTick {
            value: input.borrow().value.get(),
            now: Instant::from(SystemTime::now()),
        },
        |_, ()| {},
    );
    let presentation = settings.clone();
    let now_label = message("soft.relative_time.now", "now");
    let yesterday = message("soft.relative_time.yesterday", "yesterday");
    let display = memo(move || {
        let TimestampClock::Running { now } = clock.get();
        timestamp_presentation(&presentation.borrow(), now, now_label, yesterday)
    });
    Timestamp {
        text: text(display)
            .kind(TextKind::Supporting)
            .color(TextColor::Secondary),
        clock,
        settings,
        details: true,
        entries: Vec::new(),
        live: false,
    }
}

struct TimestampSettings {
    value: Value<Instant>,
    format: TimestampFormat,
    threshold: Duration,
    zone: TimeZone,
    show_zone: bool,
}

impl Timestamp {
    /// Select the timestamp format.
    pub fn format(self, format: TimestampFormat) -> Self {
        self.settings.borrow_mut().format = format;
        self
    }
    /// Set the relative window for automatic format.
    pub fn auto_threshold(self, threshold: Duration) -> Self {
        self.settings.borrow_mut().threshold = threshold;
        self
    }
    /// Enable or disable the details surface.
    pub fn hover_card(mut self, on: bool) -> Self {
        self.details = on;
        self
    }
    /// Supply the rows shown in the details surface.
    pub fn entries(mut self, entries: Vec<TimestampEntry>) -> Self {
        self.entries = entries;
        self
    }
    /// Set the wall-clock offset supplied by the application.
    pub fn time_zone(self, zone: TimeZone) -> Self {
        self.settings.borrow_mut().zone = zone;
        self
    }
    /// Append the time zone label to time formats.
    pub fn show_time_zone(self, on: bool) -> Self {
        self.settings.borrow_mut().show_zone = on;
        self
    }
    /// Update the relative display while mounted.
    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }
    /// Select the typography role.
    pub fn kind(mut self, kind: TextKind) -> Self {
        self.text = self.text.kind(kind);
        self
    }
    /// Override the text size.
    pub fn size(mut self, size: TextSize) -> Self {
        self.text = self.text.size(size);
        self
    }
    /// Override the text color.
    pub fn color(mut self, color: TextColor) -> Self {
        self.text = self.text.color(color);
        self
    }
    /// Override the text weight.
    pub fn weight(mut self, weight: TextWeight) -> Self {
        self.text = self.text.weight(weight);
        self
    }
}

impl NodeBuilder for Timestamp {
    fn node_mut(&mut self) -> &mut Node {
        self.text.node_mut()
    }
}

impl IntoView for Timestamp {
    fn into_view(self) -> View {
        let Self {
            text,
            clock,
            settings,
            details,
            entries,
            live,
        } = self;
        let settings = settings.borrow();
        let value = settings.value;
        let format = settings.format;
        let zone = settings.zone.clone();
        let relative = matches!(
            format,
            TimestampFormat::Relative | TimestampFormat::RelativeShort | TimestampFormat::Auto
        );
        if live && relative {
            clock.send(TimestampEvent::Tick);
        } else {
            clock.send(TimestampEvent::Refresh);
        }
        let full_zone = zone.clone();
        let full = move || {
            calendar_text(
                value.get(),
                TimestampFormat::DateTime,
                &full_zone,
                true,
                true,
            )
        };
        let text = if relative { text.name(full) } else { text };
        if details && (relative || !entries.is_empty()) {
            // Standard tooltip fallback until Soft's interactive hover card is integrated.
            tooltip(move || {
                if entries.is_empty() {
                    return calendar_text(
                        value.get(),
                        TimestampFormat::DateTime,
                        &zone,
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
                            InstantFormat::SystemDateTime => {
                                (TimestampFormat::SystemDateTime, false)
                            }
                            InstantFormat::SystemTime => (TimestampFormat::SystemTime, false),
                            InstantFormat::UnixSeconds => (TimestampFormat::UnixSeconds, false),
                        };
                        let value = calendar_text(
                            value.get(),
                            format,
                            entry.zone.as_ref().unwrap_or(&zone),
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
            })
            .attach(text.focusable(true))
        } else {
            text.into_view()
        }
    }
}

fn difference(now: Instant, then: Instant) -> i64 {
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
    instant: Instant,
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
    let (year, month, day) = civil_from_days(days);
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

// Gregorian civil dates expressed as March-based 400-year cycles.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let month = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month + 2) / 5 + day - 1;
    era * 146_097 + year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let march_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * march_month + 2) / 5 + 1;
    let month = if march_month < 10 {
        march_month + 3
    } else {
        march_month - 9
    };
    (era * 400 + year_of_era + i64::from(month <= 2), month, day)
}

#[derive(Clone, PartialEq)]
enum TimestampClock {
    Running { now: Instant },
}
struct TimestampTick {
    value: Instant,
    now: Instant,
}
impl Machine for TimestampTick {
    type State = TimestampClock;
    type Event = TimestampEvent;
    type Effect = ();
    fn transition(
        &self,
        _: &TimestampClock,
        event: TimestampEvent,
    ) -> Step<TimestampClock, TimestampEvent, ()> {
        let age = difference(self.now, self.value).unsigned_abs();
        let delay = match age {
            0..=59 => 1,
            60..=3599 => 30,
            3600..=86399 => 60,
            _ => 300,
        };
        let step = Step::to(TimestampClock::Running { now: self.now });
        match event {
            TimestampEvent::Refresh => step,
            TimestampEvent::Tick => step.after(Duration::from_secs(delay), TimestampEvent::Tick),
        }
    }
}

enum TimestampEvent {
    Refresh,
    Tick,
}
fn timestamp_presentation(
    settings: &TimestampSettings,
    now: Instant,
    now_label: Value<String>,
    yesterday: Value<String>,
) -> String {
    let instant = settings.value.get();
    let diff = difference(now, instant);
    let format = if settings.format == TimestampFormat::Auto {
        if diff.unsigned_abs() <= settings.threshold.as_secs() {
            TimestampFormat::Relative
        } else {
            TimestampFormat::DateTime
        }
    } else {
        settings.format
    };
    match format {
        TimestampFormat::Relative | TimestampFormat::RelativeShort => relative_text(
            diff,
            format == TimestampFormat::RelativeShort,
            now_label,
            yesterday,
        ),
        _ => calendar_text(instant, format, &settings.zone, settings.show_zone, false),
    }
}
