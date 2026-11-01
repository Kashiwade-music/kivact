//! Role: Owner boundaries for context, theme, locale and messages.
//!
//! Owner boundaries that wrap a child:
//! [`provide`], [`provide_theme`], [`provide_locale`],
//! [`provide_messages`] (exported by `kivact::view`)
//! and [`provide_direction`], plus [`mount_in`], the one place that
//! runs a child's setup in a fresh owner (used by these, the flow branches
//! and rows, `error_boundary` and the App root).
//!
//! Invariants:
//! - A boundary creates its own child owner; what it registers there (a
//!   context, a theme, a locale, messages) is visible to the child's subtree only, never to
//!   siblings mounted after it.
//! - The child's setup runs in a `Setup` frame of that owner (untracked);
//!   a panic propagates.
//! - The child closure runs when the view is mounted (inside the parent's
//!   setup or a flow's effect), never when the description is built.

use std::panic::Location;
use std::rc::Rc;

use crate::i18n::{Locale, Messages};
use kivact_reactive::{IntoValue, Owner, Runtime};
use kivact_style::Theme;
use kivact_wire::NodeId;

use crate::view::descriptor::Mcx;
use crate::view::descriptor::{IntoView, View, view_fn};

/// Run `build` as the setup of `owner` (a `Setup` frame; a panic
/// propagates) and mount the view it returns under `owner`.
pub(crate) fn mount_in(rt: &Rc<Runtime>, owner: Owner, build: impl FnOnce() -> View) -> NodeId {
    let view = owner.with_setup(build);
    Mcx { rt, owner }.mount_child(view)
}

/// Provide `value` to the subtree built by `child` (Solid's
/// `<Context.Provider>`): `child` is set up in a new owner that holds the
/// context, so `context::<T>()` finds it inside `child` (in setup and in
/// handlers) but not in siblings. The value is kept once (in an `Rc`) and
/// each `context::<T>()` returns a clone: a struct of Copy handles is
/// copied, and a value that is costly to clone can be provided as an `Rc`.
///
/// ```ignore
/// provide(BoardCx { tasks, selected }, || column((toolbar(), lanes())))
/// ```
#[track_caller]
pub fn provide<T: Clone + 'static, V: IntoView>(
    value: T,
    child: impl FnOnce() -> V + 'static,
) -> View {
    let at = Location::caller();
    view_fn(move |cx| {
        let owner = cx.owner.child(at);
        owner.provide(value);
        mount_in(cx.rt, owner, move || child().into_view())
    })
}

/// Use `theme` for the subtree built by `child`. Pass a `Signal<Theme>` (or
/// a closure, a memo) to switch it at runtime: every style re-resolves its
/// paint and nothing is remounted. Siblings keep the theme from above.
///
/// It makes no node, so it colors nothing by itself: text inherits its
/// color from its ancestors (as CSS's `color`), and text under no colored
/// node takes the App theme's. Give the subtree's top node the theme's
/// colors (`.background(ColorToken::Surface).color(ColorToken::Text)`, or
/// a kit's tokens), as a page sets them on its body.
///
/// ```ignore
/// let dark = signal(Theme::dark());
/// provide_theme(dark, || settings_panel())
/// ```
#[track_caller]
pub fn provide_theme<V: IntoView>(
    theme: impl IntoValue<Theme>,
    child: impl FnOnce() -> V + 'static,
) -> View {
    let at = Location::caller();
    let theme = theme.into_prop();
    view_fn(move |cx| {
        let owner = cx.owner.child(at);
        let value = theme.into_value(owner, at);
        Theme::install(owner, value);
        mount_in(cx.rt, owner, move || child().into_view())
    })
}

/// Use `locale` for the subtree built by `child` (like
/// [`provide_theme`]). Pass a `Signal<Locale>`
/// (or a closure, a memo) to switch it at runtime: everything that reads
/// [`locale`](crate::i18n::locale) below follows. Siblings keep the locale from
/// above.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::i18n::{Locale, locale};
/// use kivact::view::{provide_locale};
/// use kivact::prelude::*;
///
/// fn quote() -> impl IntoView {
///     // The quotation is French whatever the rest of the App speaks.
///     provide_locale(Locale::new("fr-FR"), || {
///         let locale = locale();
///         text(move || locale.get().tag().to_owned())
///     })
/// }
/// ```
#[track_caller]
pub fn provide_locale<V: IntoView>(
    locale: impl IntoValue<Locale>,
    child: impl FnOnce() -> V + 'static,
) -> View {
    let at = Location::caller();
    let locale = locale.into_prop();
    view_fn(move |cx| {
        let owner = cx.owner.child(at);
        let value = locale.into_value(owner, at);
        kivact_i18n::install_locale(owner, value);
        mount_in(cx.rt, owner, move || child().into_view())
    })
}

/// Use `messages` for the subtree built by `child`, on top of the messages
/// above it: a key `messages` sets is replaced there, every other key
/// keeps the text in effect outside. Pass a `Signal<Messages>` (or a
/// closure, a memo) to switch it at runtime: every
/// [`message`](crate::i18n::message) below follows. Siblings keep the messages
/// from above.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::i18n::{Messages, message};
/// use kivact::view::{provide_messages};
/// use kivact::prelude::*;
///
/// fn confirm() -> impl IntoView {
///     provide_messages(Messages::new().with("dialog.close", "Dismiss"), || {
///         text(message("dialog.close", "Close"))
///     })
/// }
/// ```
#[track_caller]
pub fn provide_messages<V: IntoView>(
    messages: impl IntoValue<Messages>,
    child: impl FnOnce() -> V + 'static,
) -> View {
    let at = Location::caller();
    let messages = messages.into_prop();
    view_fn(move |cx| {
        let owner = cx.owner.child(at);
        let own = messages.into_value(owner, at);
        // Layered over the messages `owner` inherits from `cx.owner`.
        kivact_i18n::install_layered_messages(owner, own, at);
        mount_in(cx.rt, owner, move || child().into_view())
    })
}

/// The reading direction of a subtree (the HTML `dir` attribute, base-ui's
/// `DirectionProvider`). Named apart from [`Direction`](crate::style::Direction),
/// the flex direction of a box.
///
/// Direction-aware behavior reads it once at setup with
/// [`current_direction`]: a horizontal slider's track runs from the right
/// and its arrow keys mirror. Text layout and the standard components'
/// rows do not mirror.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReadingDirection {
    /// Left to right.
    #[default]
    Ltr,
    /// Right to left.
    Rtl,
}

/// Build `child` with `dir` as the reading direction of its subtree.
///
/// ```ignore
/// provide_direction(ReadingDirection::Rtl, || arabic_settings())
/// ```
#[track_caller]
pub fn provide_direction<V: IntoView>(
    dir: ReadingDirection,
    child: impl FnOnce() -> V + 'static,
) -> View {
    provide(dir, child)
}

/// The reading direction provided above the current owner;
/// [`ReadingDirection::Ltr`] without one. Setup, handlers and effects.
#[track_caller]
pub fn current_direction() -> ReadingDirection {
    kivact_reactive::try_context::<ReadingDirection>().unwrap_or_default()
}
