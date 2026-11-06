//! Role: Provide locale, calendar and message values and pure text calculations.
//!
//! Applications own translation resources in [`Messages`] and identify their
//! language and region with [`Locale`]. Owner contexts and reactive message
//! readers belong to `kivact_core::i18n`; this crate holds values and pure
//! calculations, including [`interpolate`]. Regional formats, calendar names,
//! week conventions, plural rules and collation use ICU4X compiled data for
//! all supported locales. ICU implementation types stay behind this API.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod date;
mod format;
mod interpolate;
mod locale;
mod messages;
mod numbers;
mod regional;

pub use date::{CalendarDate, CalendarMonth, DateRange};
pub use format::{
    DateStyle, DateTime, FractionDigits, Grouping, HourCycle, NameWidth, NumberStyle,
    PluralCategory, PluralForms, TimeOfDay, TimeStyle,
};
pub use interpolate::interpolate;
pub use locale::{Locale, Weekday};
pub use messages::Messages;
