//! Role: Apply ICU calendar patterns, display names, plural rules, and collation.

use crate::{
    CalendarDate, DateStyle, HourCycle, Locale, NameWidth, PluralCategory, TimeOfDay, TimeStyle,
    Weekday,
};
use icu_calendar::{Date, Gregorian};
use icu_datetime::fieldsets::{T, YMD};
use icu_datetime::input::Time;
use icu_datetime::options::TimePrecision;
use icu_datetime::pattern::{
    DateTimePattern, FixedCalendarDateTimeNames, MonthNameLength, WeekdayNameLength,
};
use icu_datetime::{FixedCalendarDateTimeFormatter, NoCalendarFormatter};
use std::cmp::Ordering;
use writeable::{TryWriteable, Writeable};

impl Locale {
    /// Format a Gregorian date using the requested locale style. A year
    /// outside -9999 to 9999, which the locale calendar data does not cover,
    /// prints as ISO 8601.
    pub fn format_date(&self, date: CalendarDate, style: DateStyle) -> String {
        let Some(icu) = icu_date(date).filter(|_| style != DateStyle::Iso) else {
            return date.to_string();
        };
        let fields = match style {
            DateStyle::Short => YMD::short(),
            DateStyle::Medium => YMD::medium(),
            DateStyle::Long | DateStyle::Iso => YMD::long(),
        };
        let formatter =
            FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(self.icu().into(), fields)
                .expect("invariant: compiled ICU Gregorian date data includes locale fallback");
        formatter.format(&icu).to_string()
    }

    /// Parse ISO 8601, locale month names, or a numeric date in locale order; two-digit years use 2000 to 2099.
    pub fn parse_date(&self, text: &str) -> Option<CalendarDate> {
        let text = text.trim();
        let iso: Vec<_> = text.split('-').collect();
        if iso.len() == 3
            && iso[0].len() >= 4
            && let (Ok(year), Ok(month), Ok(day)) = (iso[0].parse(), iso[1].parse(), iso[2].parse())
        {
            return CalendarDate::checked(year, month, day);
        }
        let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
            self.icu().into(),
            YMD::short(),
        )
        .expect("invariant: compiled ICU Gregorian date data includes locale fallback");
        let date = sample_date(CalendarDate::new(2006, 11, 22));
        let formatted = formatter.format(&date);
        let pattern = formatted.pattern().write_to_string().into_owned();
        let order = date_order(&pattern);
        if let Some(date) = named_date(self, text, &pattern) {
            return Some(date);
        }
        let numbers = crate::numbers::date_fields(self, text, &pattern)?;
        if numbers.len() != 3 || order.len() != 3 {
            return None;
        }
        let mut year = 0;
        let mut month = 0;
        let mut day = 0;
        for (field, number) in order.iter().zip(numbers) {
            match field {
                'y' => {
                    year = number.parse::<i32>().ok()?;
                    if number.len() <= 2 {
                        year += 2000;
                    }
                }
                'M' => month = number.parse().ok()?,
                'd' => day = number.parse().ok()?,
                _ => return None,
            }
        }
        CalendarDate::checked(year, month, day)
    }

    /// Format a wall-clock time, returning an empty string for invalid input.
    pub fn format_time(&self, time: TimeOfDay, style: TimeStyle) -> String {
        let Ok(time) = Time::try_new(time.hour, time.minute, time.second, 0) else {
            return String::new();
        };
        let precision = if style.seconds {
            TimePrecision::Second
        } else {
            TimePrecision::Minute
        };
        let formatter = NoCalendarFormatter::try_new(
            self.icu().into(),
            T::medium().with_time_precision(precision),
        )
        .expect("invariant: compiled ICU time data includes locale fallback");
        formatter.format(&time).to_string()
    }

    /// Return the resolved locale hour cycle, including Unicode preferences.
    pub fn hour_cycle(&self) -> HourCycle {
        let formatter = NoCalendarFormatter::try_new(self.icu().into(), T::medium())
            .expect("invariant: compiled ICU time data includes locale fallback");
        let time = Time::try_new(13, 0, 0, 0).expect("invariant: valid literal time");
        let formatted = formatter.format(&time);
        let components =
            icu_datetime::provider::fields::components::Bag::from(&formatted.pattern());
        use icu_locale_core::preferences::extensions::unicode::keywords::HourCycle as IcuHourCycle;
        match components.hour_cycle {
            Some(IcuHourCycle::H11 | IcuHourCycle::H12) => HourCycle::H12,
            _ => HourCycle::H23,
        }
    }

    /// Return a stand-alone Gregorian month name, or empty for an invalid month.
    pub fn month_name(&self, month: u8, width: NameWidth) -> String {
        if !(1..=12).contains(&month) {
            return String::new();
        }
        let mut names: FixedCalendarDateTimeNames<
            Gregorian,
            icu_datetime::fieldsets::enums::DateFieldSet,
        > = FixedCalendarDateTimeNames::try_new(self.icu().into())
            .expect("invariant: compiled ICU date names include locale fallback");
        let (length, pattern) = match width {
            NameWidth::Narrow => (MonthNameLength::StandaloneNarrow, "LLLLL"),
            NameWidth::Short => (MonthNameLength::StandaloneAbbreviated, "LLL"),
            NameWidth::Long => (MonthNameLength::StandaloneWide, "LLLL"),
        };
        names
            .include_month_names(length)
            .expect("invariant: Gregorian month names are available");
        let pattern: DateTimePattern = pattern
            .parse()
            .expect("invariant: valid literal month pattern");
        names
            .with_pattern_unchecked(&pattern)
            .format(&sample_date(CalendarDate::new(2026, month, 1)))
            .try_write_to_string()
            .expect("invariant: requested month names were loaded")
            .into_owned()
    }

    /// Return a stand-alone weekday name.
    pub fn weekday_name(&self, day: Weekday, width: NameWidth) -> String {
        let mut names: FixedCalendarDateTimeNames<
            Gregorian,
            icu_datetime::fieldsets::enums::DateFieldSet,
        > = FixedCalendarDateTimeNames::try_new(self.icu().into())
            .expect("invariant: compiled ICU date names include locale fallback");
        let (length, pattern) = match width {
            NameWidth::Narrow => (WeekdayNameLength::StandaloneNarrow, "ccccc"),
            NameWidth::Short => (WeekdayNameLength::StandaloneAbbreviated, "ccc"),
            NameWidth::Long => (WeekdayNameLength::StandaloneWide, "cccc"),
        };
        names
            .include_weekday_names(length)
            .expect("invariant: weekday names are available");
        let pattern: DateTimePattern = pattern
            .parse()
            .expect("invariant: valid literal weekday pattern");
        let date = CalendarDate::new(2026, 11, 2).add_days(day.days_since(Weekday::Monday) as i32);
        names
            .with_pattern_unchecked(&pattern)
            .format(&sample_date(date))
            .try_write_to_string()
            .expect("invariant: requested weekday names were loaded")
            .into_owned()
    }

    /// Return the Unicode cardinal category; non-finite values use Other.
    pub fn plural_category(&self, n: f64) -> PluralCategory {
        let Ok(operands) = n.abs().to_string().parse::<icu_decimal::input::Decimal>() else {
            return PluralCategory::Other;
        };
        let rules = icu_plurals::PluralRules::try_new_cardinal(self.icu().into())
            .expect("invariant: compiled ICU plural rules include locale fallback");
        match rules.category_for(&operands) {
            icu_plurals::PluralCategory::Zero => PluralCategory::Zero,
            icu_plurals::PluralCategory::One => PluralCategory::One,
            icu_plurals::PluralCategory::Two => PluralCategory::Two,
            icu_plurals::PluralCategory::Few => PluralCategory::Few,
            icu_plurals::PluralCategory::Many => PluralCategory::Many,
            icu_plurals::PluralCategory::Other => PluralCategory::Other,
        }
    }

    /// Compare strings using the locale collation rules.
    pub fn compare(&self, a: &str, b: &str) -> Ordering {
        icu_collator::Collator::try_new(self.icu().into(), Default::default())
            .expect("invariant: compiled ICU collation data includes locale fallback")
            .compare(a, b)
    }
}

/// The ICU date, or `None` for a year outside ICU's -9999 to 9999.
fn icu_date(date: CalendarDate) -> Option<Date<Gregorian>> {
    Date::try_new_gregorian(date.year(), date.month(), date.day()).ok()
}

/// The ICU date of a sample day the formatter reads names or patterns from.
fn sample_date(date: CalendarDate) -> Date<Gregorian> {
    icu_date(date).expect("invariant: sample dates are within ICU's year range")
}

fn date_order(pattern: &str) -> Vec<char> {
    let mut order = Vec::new();
    let mut quoted = false;
    for c in pattern.chars() {
        if c == '\'' {
            quoted = !quoted;
        }
        if quoted {
            continue;
        }
        let field = match c {
            'y' | 'u' => 'y',
            'M' | 'L' => 'M',
            'd' => 'd',
            _ => continue,
        };
        if !order.contains(&field) {
            order.push(field);
        }
    }
    order
}

/// Parse the locale's own stand-alone month names without an English table.
fn named_date(locale: &Locale, text: &str, pattern: &str) -> Option<CalendarDate> {
    let lower = text.to_lowercase();
    for month in 1..=12 {
        for width in [NameWidth::Long, NameWidth::Short] {
            let name = locale.month_name(month, width).to_lowercase();
            if name.is_empty() || !name.chars().any(char::is_alphabetic) {
                continue;
            }
            let Some(start) = lower.find(&name) else {
                continue;
            };
            let end = start + name.len();
            if lower[..start]
                .chars()
                .next_back()
                .is_some_and(char::is_alphabetic)
                || lower[end..].chars().next().is_some_and(char::is_alphabetic)
            {
                continue;
            }
            let rest = format!("{} {}", &lower[..start], &lower[end..]);
            let fields = crate::numbers::date_fields(locale, &rest, pattern)?;
            let [first, second] = fields.as_slice() else {
                continue;
            };
            let (year, day) = if first.len() >= 4 {
                (first, second)
            } else {
                (second, first)
            };
            let mut parsed_year = year.parse::<i32>().ok()?;
            if year.len() <= 2 {
                parsed_year += 2000;
            }
            if let Some(date) = CalendarDate::checked(parsed_year, month, day.parse().ok()?) {
                return Some(date);
            }
        }
    }
    None
}
