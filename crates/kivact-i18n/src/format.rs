//! Role: Public options and values for locale-sensitive formatting.

use crate::CalendarDate;

/// A wall-clock time without a date or time zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeOfDay {
    /// Hour in the range 0 to 23.
    pub hour: u8,
    /// Minute in the range 0 to 59.
    pub minute: u8,
    /// Second in the range 0 to 59.
    pub second: u8,
}
/// A Gregorian date and wall-clock time without a time zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    /// Calendar date.
    pub date: CalendarDate,
    /// Wall-clock time.
    pub time: TimeOfDay,
}
/// The locale's default hour cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HourCycle {
    /// Twelve-hour clock.
    H12,
    /// Twenty-four-hour clock starting at zero.
    H23,
}
/// Decimal precision and digit grouping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberStyle {
    /// Precision after the decimal separator.
    pub fraction_digits: FractionDigits,
    /// Whether to group integer digits.
    pub grouping: Grouping,
}
/// Precision after the decimal separator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FractionDigits {
    /// Round and pad to exactly this many digits.
    Exact(u8),
    /// Round to `max` digits, retaining at least `min`.
    Range {
        /// Minimum retained digits; capped at `max`.
        min: u8,
        /// Maximum retained digits.
        max: u8,
    },
}
/// Whether to separate groups of integer digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grouping {
    /// Show no grouping separators.
    None,
    /// Follow the locale's grouping sizes and minimum digits.
    Locale,
}
/// The requested presentation of a Gregorian date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateStyle {
    /// Numeric date in locale order.
    Short,
    /// Date with an abbreviated month where the locale uses one.
    Medium,
    /// Date with the full month where the locale uses one.
    Long,
    /// ISO 8601 year, month, and day.
    Iso,
}
/// Precision of a locale-sensitive wall-clock time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeStyle {
    /// Include seconds.
    pub seconds: bool,
}
/// Width of a stand-alone calendar name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameWidth {
    /// Narrow name, which may be ambiguous.
    Narrow,
    /// Abbreviated name.
    Short,
    /// Full name.
    Long,
}
/// A Unicode cardinal plural category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluralCategory {
    /// Zero category.
    Zero,
    /// One category.
    One,
    /// Two category.
    Two,
    /// Few category.
    Few,
    /// Many category.
    Many,
    /// Other category, including values without a matching optional form.
    Other,
}
/// Message templates for the cardinal plural categories.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluralForms {
    // xtask-allow(state-fields): option: the message author's template for the zero category; absent falls back to the other template
    /// Optional zero template.
    pub zero: Option<String>,
    // xtask-allow(state-fields): option: the message author's template for the one category; absent falls back to the other template
    /// Optional one template.
    pub one: Option<String>,
    // xtask-allow(state-fields): option: the message author's template for the two category; absent falls back to the other template
    /// Optional two template.
    pub two: Option<String>,
    // xtask-allow(state-fields): option: the message author's template for the few category; absent falls back to the other template
    /// Optional few template.
    pub few: Option<String>,
    // xtask-allow(state-fields): option: the message author's template for the many category; absent falls back to the other template
    /// Optional many template.
    pub many: Option<String>,
    /// Required fallback template.
    pub other: String,
}

kivact_runtime::constant_value!(
    TimeOfDay,
    DateTime,
    HourCycle,
    NumberStyle,
    FractionDigits,
    Grouping,
    DateStyle,
    TimeStyle,
    NameWidth,
    PluralCategory,
    PluralForms
);
