//! Role: A calendar grid behavior attaches month, week, day and month-paging parts.
//!
//! [`use_calendar_grid`] takes the shown month (the first of the months drawn
//! side by side), the caller's [`CalendarSelection`] (one day or a range) and
//! a [`CalendarOptions`]. The caller draws everything: for each shown month
//! a container with [`grid`](CalendarGridApi::grid), a [`row`](CalendarGridApi::row)
//! per week and a [`cell`](CalendarGridApi::cell) per day (a month's days and
//! weeks are [`CalendarMonth::weeks`]), and the buttons that page the months.
//! The grid keeps the keyboard focus, the days that cannot be chosen and the
//! first click of a range; it creates no node.
//!
//! One day of the shown months holds the tab stop: the day the keyboard or a
//! click last moved to, else the selected day (the start of the range), else
//! the policy's first focus, else the first enabled day. A day of a
//! neighbouring month drawn in the first or last week (a cell whose date lies
//! outside the month passed to `cell`) can be clicked but takes no focus.
//! On a focused day the arrows move by a day or a week, Home and End go to
//! the first and last day of the week, PageUp and PageDown move a month and
//! Shift with them a year, and Enter, Space or a click chooses the day (a
//! click makes the day the tab stop but does not move the focus). A
//! move skips disabled days and a move into a month that is not shown pages
//! the shown months to it and keeps the focus. A disabled day takes neither
//! focus nor a choice.
//!
//! With [`CalendarSelection::Single`] a choice writes `Some(day)`. With
//! [`CalendarSelection::Range`] the first choice writes the range of that one
//! day and the second completes it, in either order. Escape while the second
//! choice is awaited puts back the range from before the first choice; a write
//! to the range binding from elsewhere starts the next choice over. The
//! policy's `on_commit` runs when a choice is complete: each choice of a single
//! day, the second choice of a range.
//!
//! ```no_run
//! # mod kivact {
//! #     //! Role: Supply facade paths for this documentation example.
//! # pub use kivact_core::{view, state, style};
//! # pub mod prelude {
//! # //! Role: Supply the canonical imports used by this documentation example.
//! # pub use kivact_core::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Signal, Value, binding, effect, memo, on_cleanup, on_mount, signal};
//! # pub use kivact_core::view::{EditPhase, Component, IntoView, NodeBuilder, Orientation, Role, column, for_each, node, node_ref, row, show, text};
//! # pub use kivact_core::style::{ColorToken, Cursor, Style, style};
//! # }
//! #     pub extern crate kivact_interaction as interaction;
//! # }
//! use kivact::view::{column, node, row, text};
//! use kivact::i18n::{CalendarDate, CalendarMonth};
//! use kivact::interaction::{CalendarOptions, CalendarSelection, use_calendar_grid};
//! use kivact::prelude::*;
//!
//! let day = signal(None::<CalendarDate>);
//! let month = signal(CalendarMonth::new(2026, 10));
//! let calendar = use_calendar_grid(month, CalendarSelection::Single(day.into_binding()), CalendarOptions::new());
//! let shown = CalendarMonth::new(2026, 10);
//! let weeks = shown.weeks(calendar.week_start().get_untracked(), 0);
//! let view = column((
//!     row((
//!         node().part(calendar.previous_month()).name("Previous month"),
//!         node().part(calendar.next_month()).name("Next month"),
//!     )),
//!     column(weeks.into_iter().map(|week| {
//!         row(week.into_iter().map(|date| {
//!             node().part(calendar.cell(shown, date)).child(text(date.day().to_string()))
//!         }).collect::<Vec<_>>())
//!         .part(calendar.row())
//!     }).collect::<Vec<_>>())
//!     .part(calendar.grid()),
//! ));
//! ```

mod handle;
mod policy;
mod state;

pub use handle::{CalendarDayState, CalendarGridApi, CalendarSpan, use_calendar_grid};
pub use policy::{CalendarOptions, CalendarSelection};
