//! Role: Public locale and message names of the facade.
//!
//! Value types and interpolation come from `kivact-i18n`; owner context
//! readers live in the core view boundary.

#[doc(inline)]
pub use crate::view::boundary::{locale, message, message_with, provide_locale, provide_messages};
#[doc(inline)]
pub use kivact_i18n::{
    CalendarDate, CalendarMonth, DateRange, DateStyle, DateTime, FractionDigits, Grouping,
    HourCycle, Locale, Messages, NameWidth, NumberStyle, PluralCategory, PluralForms, TimeOfDay,
    TimeStyle, Weekday, interpolate,
};
