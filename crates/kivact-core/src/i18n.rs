//! Role: Public locale and message names of the facade.
//!
//! The implementation is the `kivact-i18n` crate; this module re-exports the
//! names an app or a component author reads (`locale`, `message`, ...). The
//! functions the view root and boundaries use to provide them
//! (`install_locale`, `install_messages`, `install_layered_messages`) stay in
//! `kivact_i18n` and are not part of the public facade: apps provide a locale
//! or messages with `view::provide_locale` and `view::provide_messages`.

#[doc(inline)]
pub use kivact_i18n::{
    CalendarDate, CalendarMonth, DateRange, DateStyle, DateTime, FractionDigits, Grouping,
    HourCycle, Locale, Messages, NameWidth, NumberStyle, PluralCategory, PluralForms, TimeOfDay,
    TimeStyle, Weekday, interpolate, locale, message, message_with,
};
