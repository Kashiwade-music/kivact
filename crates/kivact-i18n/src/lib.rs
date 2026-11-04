//! Role: Provide locale, calendar and message values and pure text calculations.
//!
//! Applications own translation resources in [`Messages`] and identify their
//! language and region with [`Locale`]. Owner contexts and reactive message
//! readers belong to `kivact_core::i18n`; this crate holds values and pure
//! calculations, including [`interpolate`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod date;
mod interpolate;
mod locale;
mod messages;

pub use date::{CalendarDate, CalendarMonth, DateRange};
pub use locale::{Locale, Weekday};
pub use messages::Messages;

pub use interpolate::interpolate;
