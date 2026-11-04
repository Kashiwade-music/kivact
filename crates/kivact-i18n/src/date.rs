//! Role: Gregorian dates, months and date ranges with the arithmetic calendars need.

use crate::Weekday;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// A day of the proleptic Gregorian calendar, free of time zones and date
/// libraries.
///
/// A date is always a real day: [`checked`](Self::checked) refuses an
/// impossible one and [`new`](Self::new) moves it to the nearest real one. It
/// compares in calendar order and prints as ISO 8601 (`2026-10-03`). The
/// arithmetic is exact for years within about a million of 0.
///
/// ```
/// use kivact::i18n::{CalendarDate, Weekday};
///
/// let day = CalendarDate::checked(2026, 1, 31).unwrap();
/// assert_eq!(day.add_months(1), CalendarDate::new(2026, 2, 28));
/// assert_eq!(day.add_days(1).to_string(), "2026-02-01");
/// assert_eq!(day.weekday(), Weekday::Saturday);
/// assert_eq!(CalendarDate::checked(2026, 2, 29), None);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    year: i32,
    month: u8,
    day: u8,
}

/// A month of a year.
///
/// ```
/// use kivact::i18n::CalendarMonth;
///
/// assert_eq!(CalendarMonth::new(2026, 12).add_months(1), CalendarMonth::new(2027, 1));
/// assert_eq!(CalendarMonth::new(2024, 2).days(), 29);
/// // October 2026 starts on a Thursday: a Monday-first grid has 5 weeks.
/// let weeks = CalendarMonth::new(2026, 10).weeks(kivact::i18n::Weekday::Monday, 0);
/// assert_eq!(weeks.len(), 5);
/// assert_eq!(weeks[0][3].to_string(), "2026-10-01");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarMonth {
    year: i32,
    month: u8,
}

/// An inclusive span of days; the start is never after the end.
///
/// ```
/// use kivact::i18n::{CalendarDate, DateRange};
///
/// let range = DateRange::new(CalendarDate::new(2026, 5, 9), CalendarDate::new(2026, 5, 1));
/// assert_eq!(range.start(), CalendarDate::new(2026, 5, 1));
/// assert!(range.contains(CalendarDate::new(2026, 5, 5)));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DateRange {
    start: CalendarDate,
    end: CalendarDate,
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn month_length(year: i32, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 31,
    }
}

/// Days from 1970-01-01 to the given civil date.
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date `days` after 1970-01-01.
fn civil_from_days(days: i64) -> CalendarDate {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    let year = yoe + era * 400 + i64::from(month <= 2);
    CalendarDate {
        year: year.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        month,
        day,
    }
}

impl CalendarDate {
    /// The date, or `None` when the month is not 1 to 12 or the day is not in
    /// that month.
    pub fn checked(year: i32, month: u8, day: u8) -> Option<Self> {
        let real = (1..=12).contains(&month) && (1..=month_length(year, month)).contains(&day);
        real.then_some(Self { year, month, day })
    }

    /// The date with the month moved into 1 to 12 and the day into the month,
    /// for values that are known to be near a real date.
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        let month = month.clamp(1, 12);
        Self {
            year,
            month,
            day: day.clamp(1, month_length(year, month)),
        }
    }

    /// Today's date in UTC, read from the system clock.
    pub fn today() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        civil_from_days((secs / 86_400) as i64)
    }

    /// The year (astronomical numbering, so 1 BC is 0).
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, 1 (January) to 12.
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, from 1.
    pub fn day(self) -> u8 {
        self.day
    }

    /// Days since 1970-01-01 (negative before).
    pub fn to_days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    /// The date `days` after 1970-01-01.
    pub fn from_days(days: i64) -> Self {
        civil_from_days(days)
    }

    /// The date `days` later (earlier when negative).
    pub fn add_days(self, days: i32) -> Self {
        civil_from_days(self.to_days() + i64::from(days))
    }

    /// The date `months` later with the same day, or the last day of a
    /// shorter month.
    pub fn add_months(self, months: i32) -> Self {
        let moved = self.month_of().add_months(months);
        Self::new(moved.year, moved.month, self.day)
    }

    /// The day of the week.
    pub fn weekday(self) -> Weekday {
        // 1970-01-01 was a Thursday.
        Weekday::Monday.days_after((self.to_days() + 3).rem_euclid(7) as usize)
    }

    /// The number of days in this date's month.
    pub fn days_in_month(self) -> u8 {
        month_length(self.year, self.month)
    }

    /// The first day of this date's month.
    pub fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    /// The month this date lies in.
    pub fn month_of(self) -> CalendarMonth {
        CalendarMonth {
            year: self.year,
            month: self.month,
        }
    }
}

impl fmt::Display for CalendarDate {
    /// ISO 8601: `2026-10-03`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl CalendarMonth {
    /// A month; the month number is moved into 1 to 12.
    pub fn new(year: i32, month: u8) -> Self {
        Self {
            year,
            month: month.clamp(1, 12),
        }
    }

    /// The year.
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, 1 (January) to 12.
    pub fn month(self) -> u8 {
        self.month
    }

    /// The month `months` later (earlier when negative).
    pub fn add_months(self, months: i32) -> Self {
        let total = i64::from(self.year) * 12 + i64::from(self.month) - 1 + i64::from(months);
        Self {
            year: total
                .div_euclid(12)
                .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
            month: (total.rem_euclid(12) + 1) as u8,
        }
    }

    /// The first day of the month.
    pub fn first_day(self) -> CalendarDate {
        CalendarDate {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }

    /// The number of days in the month.
    pub fn days(self) -> u8 {
        month_length(self.year, self.month)
    }

    /// The weeks that show the month, each of seven days from `first`: the
    /// days of the neighbouring months fill the first and last week. There
    /// are at least `at_least` weeks (a grid of a fixed height asks for 6);
    /// the extra weeks continue into the next month.
    pub fn weeks(self, first: Weekday, at_least: usize) -> Vec<[CalendarDate; 7]> {
        let first_day = self.first_day();
        let lead = first_day.weekday().days_since(first);
        let count = (lead + usize::from(self.days())).div_ceil(7).max(at_least);
        let start = first_day.add_days(-(lead as i32));
        (0..count)
            .map(|week| std::array::from_fn(|day| start.add_days((week * 7 + day) as i32)))
            .collect()
    }

    /// The months from `self` to `later` (negative when earlier).
    pub fn months_until(self, later: CalendarMonth) -> i32 {
        ((i64::from(later.year) - i64::from(self.year)) * 12 + i64::from(later.month)
            - i64::from(self.month)) as i32
    }
}

impl DateRange {
    /// The range between two dates in either order.
    pub fn new(a: CalendarDate, b: CalendarDate) -> Self {
        if a <= b {
            Self { start: a, end: b }
        } else {
            Self { start: b, end: a }
        }
    }

    /// The first day.
    pub fn start(self) -> CalendarDate {
        self.start
    }

    /// The last day.
    pub fn end(self) -> CalendarDate {
        self.end
    }

    /// Whether `date` lies in the range, ends included.
    pub fn contains(self, date: CalendarDate) -> bool {
        self.start <= date && date <= self.end
    }
}
