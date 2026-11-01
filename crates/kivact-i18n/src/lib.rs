//! Role: Locale and message text management.
//!
//! Locale and message texts: which language and region the App speaks, and
//! the default texts of components, replaceable by the App.
//!
//! The App owns its translations and the language switch. Kivact carries
//! the [`Locale`] down the view tree (like the theme), and a component
//! that shows a text of its own reads it with [`message`], under a stable
//! key, so that the App can replace it with `App::messages` or
//! `kivact::view::provide_messages`.
//!
//! ```no_run
//! use kivact::i18n::{Locale, Messages};
//! use kivact::prelude::*;
//!
//! fn main() -> kivact::Result<()> {
//!     let ja = Messages::new().with("dialog.close", "閉じる");
//!     App::new("Notes")
//!         .locale(Locale::new("ja-JP"))
//!         .messages(ja)
//!         .run(|| text("こんにちは"))
//! }
//! ```
//!
//! A component's author reads the text in setup:
//!
//! ```
//! use kivact::i18n::message;
//! use kivact::prelude::*;
//!
//! fn close_button() -> Node {
//!     // "Close" unless the App replaced the key "dialog.close".
//!     let label = message("dialog.close", "Close");
//!     node().role(Role::Button).name(label).focusable(true).child(text(label))
//! }
//! ```
//!
//! An App that names no locale speaks the system's (`ja-JP` on a Japanese
//! system, `en-US` where the system names none) and follows it when the
//! user changes it while the App runs; the host notices such a change when
//! the window regains the focus, and on Windows when the system says its
//! settings changed. An App that sets `App::locale` keeps it and ignores
//! the host's changes (so does `kivact::test`'s `RenderOptions::locale`:
//! `Some` wins over the host locale).
//!
//! The host locale arrives on the wire as `HostEnv::locale`
//! (`Option<String>`, BCP 47; `None` reads as `en-US`), in `HostInfo::env`
//! at start and in `HostEvent::EnvChanged` when it changes. A root without
//! `App::locale` reads [`locale`] as a memo of that field; one with it
//! holds the App's locale as a constant. A subtree under
//! `kivact::view::provide_locale` keeps its own locale and gets no host change.
//!
//! A component passes the `Value<String>` it read to `name`, `text` or
//! `placeholder` as it is and makes no effect of its own for the text. A
//! component with texts lists their keys and defaults in its rustdoc. The core holds no
//! translation resources and no plural rules: the App builds its
//! [`Messages`] (the part i18next or solid-i18n would play), and a text
//! that needs plural forms gets one key per form. Of the regional
//! formats, only [`Locale::first_day_of_week`] exists so far, beside the
//! [`CalendarDate`] a calendar works on; date and number formats are later
//! work.
//!
//! Invariants:
//! - The App's root always provides a locale and messages; `locale` and
//!   `message` read the nearest one above the caller, so a subtree
//!   provided by `kivact::view::provide_locale` or `kivact::view::provide_messages` sees its own
//!   and nothing else does.
//! - A subtree's messages are layered over the messages above it: a key
//!   the subtree does not set keeps the outer text. `kivact::view::provide_locale`
//!   replaces the locale only and leaves the messages as they are.
//! - The root creates a `Signal<Locale>` and a `Signal<Messages>` and
//!   provides them as `Value`s (`LocaleCtx`, `MessagesCtx`, the shape of
//!   the theme's `ThemeCtx`); readers search up from the nearest owner.
//!   `provide_locale` and `provide_messages` (`kivact::view`) are
//!   boundaries of the same shape as `provide_theme` and take a `Signal`,
//!   a closure or a memo.
//! - `message` is reactive: switching the locale or the messages at
//!   runtime updates every text read from it, remounting nothing.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod context;
mod date;
mod locale;
mod messages;

pub use date::{CalendarDate, CalendarMonth, DateRange};
pub use locale::{Locale, Weekday};
pub use messages::Messages;

pub use context::{
    install_layered_messages, install_locale, install_messages, interpolate, locale, message,
    message_with,
};
