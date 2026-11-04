//! Role: Public locale and message names of the facade.
//!
//! Value types and interpolation come from `kivact-i18n`; owner context
//! readers live in the core view boundary.

#[doc(inline)]
pub use crate::view::boundary::{locale, message, message_with};
#[doc(inline)]
pub use kivact_i18n::{
    CalendarDate, CalendarMonth, DateRange, Locale, Messages, Weekday, interpolate,
};
