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

use kivact_i18n::{Locale, Messages, interpolate};
use kivact_runtime::{IntoValue, Owner, Prop, Runtime, Value, memo};
use kivact_style::Theme;
use kivact_types::NodeId;

use crate::view::descriptor::Mcx;
use crate::view::descriptor::{IntoView, View, view_fn};
use kivact_types::WindowId;

/// Run `build` as the setup of `owner` (a `Setup` frame; a panic
/// propagates) and mount the view it returns under `owner` in `window`.
pub(crate) fn mount_in(
    rt: &Rc<Runtime>,
    owner: Owner,
    window: WindowId,
    build: impl FnOnce() -> View,
) -> NodeId {
    let view = owner.with_setup(build);
    Mcx { rt, owner, window }.mount_child(view)
}

/// Build a component once when mounted, within its own setup and cleanup owner.
///
/// The initializer inherits context from the mounting parent. Its signals, effects,
/// and cleanup callbacks belong to this component and are disposed with it.
/// The boundary adds a transparent fragment without a visible layout box.
/// Use a factory for content that must be constructed again on later mounts.
#[track_caller]
pub fn component<V: IntoView>(build: impl FnOnce() -> V + 'static) -> View {
    let at = Location::caller();
    view_fn(move |cx| {
        let owner = cx.owner.child(at);
        let fragment = cx.fragment();
        let child = mount_in(cx.rt, owner, cx.window, move || build().into_view());
        cx.append(fragment, child);
        fragment
    })
}

/// Provide `value` to the subtree built by `child` (Solid's
/// `<Context.Provider>`): `child` is set up in a new owner that holds the
/// context, so `use_context::<T>()` finds it inside `child` (in setup and in
/// handlers) but not in siblings. The value is kept once (in an `Rc`) and
/// each `use_context::<T>()` returns a `Value<T>`; read it with `get` or `with`.
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
        let value = owner.constant(value, at);
        owner.provide(value);
        let fragment = cx.fragment();
        let child = mount_in(cx.rt, owner, cx.window, move || child().into_view());
        cx.append(fragment, child);
        fragment
    })
}

/// Use `theme` for the subtree built by `child`. Pass a `Signal<Theme>` (or
/// a closure, a memo) to switch it at runtime: every style re-resolves its
/// paint and nothing is remounted. Siblings keep the theme from above.
///
/// Its transparent box colors nothing by itself: text inherits its
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
        kivact_style::internal::install_theme(owner, value);
        let fragment = cx.fragment();
        let child = mount_in(cx.rt, owner, cx.window, move || child().into_view());
        cx.append(fragment, child);
        fragment
    })
}

/// Use `locale` for the subtree built by `child` (like
/// [`provide_theme`]). Pass a `Signal<Locale>`
/// (or a closure, a memo) to switch it at runtime: everything that reads
/// [`locale`](crate::i18n::use_locale) below follows. Siblings keep the locale from
/// above.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::i18n::{Locale, use_locale};
/// use kivact::i18n::provide_locale;
/// use kivact::{App};
/// use kivact::view::{IntoView, NodeBuilder, text};
///
/// fn quote() -> impl IntoView {
///     // The quotation is French whatever the rest of the App speaks.
///     provide_locale(Locale::new("fr-FR"), || {
///         let locale = use_locale();
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
        install_locale(owner, value);
        let fragment = cx.fragment();
        let child = mount_in(cx.rt, owner, cx.window, move || child().into_view());
        cx.append(fragment, child);
        fragment
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
/// use kivact::i18n::provide_messages;
/// use kivact::view::{IntoView, NodeBuilder, text};
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
        install_layered_messages(owner, own, at);
        let fragment = cx.fragment();
        let child = mount_in(cx.rt, owner, cx.window, move || child().into_view());
        cx.append(fragment, child);
        fragment
    })
}

/// The reading direction of a subtree (the HTML `dir` attribute, base-ui's
/// `DirectionProvider`). Named apart from [`Direction`](crate::style::Direction),
/// the flex direction of a box.
///
/// Direction-aware behavior reads it once at setup with
/// [`use_direction`]: a horizontal slider's track runs from the right
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
pub fn use_direction() -> Value<ReadingDirection> {
    let owner = Owner::ambient("use_direction", Location::caller());
    owner
        .context::<Value<ReadingDirection>>()
        .unwrap_or_else(|| owner.constant(ReadingDirection::Ltr, Location::caller()))
}

/// Context entry for the locale of a subtree.
#[derive(Clone, Copy)]
pub(crate) struct LocaleCtx(pub(crate) Value<Locale>);

/// Context entry for the messages in effect in a subtree.
#[derive(Clone, Copy)]
pub(crate) struct MessagesCtx(pub(crate) Value<Messages>);

/// The locale provided to `owner`'s subtree (the App's root provides one).
pub(crate) fn locale_value(owner: Owner) -> Value<Locale> {
    owner
        .context::<LocaleCtx>()
        .map(|c| c.0)
        .expect("invariant: the App's root provides the locale")
}

/// The messages in effect for `owner`'s subtree (the App's root provides
/// them).
pub(crate) fn messages_value(owner: Owner) -> Value<Messages> {
    owner
        .context::<MessagesCtx>()
        .map(|c| c.0)
        .expect("invariant: the App's root provides the messages")
}

/// Provide `locale` to `owner`'s subtree, replacing any inherited locale.
///
/// The App's root and `kivact::view::provide_locale` use it; [`locale`]
/// reads the nearest one above the caller.
pub(crate) fn install_locale(owner: Owner, locale: Value<Locale>) {
    owner.provide(LocaleCtx(locale));
}

/// Provide `messages` to `owner`'s subtree, replacing the inherited ones
/// (the App's root uses it).
pub(crate) fn install_messages(owner: Owner, messages: Value<Messages>) {
    owner.provide(MessagesCtx(messages));
}

/// Provide `messages` to `owner`'s subtree layered over the messages
/// `owner` inherits: a key found in `messages` wins, others fall back.
/// Both sides stay reactive: a switch on either reaches the readers.
///
/// `at` is the creation site recorded for the layered value (pass the
/// caller's `Location::caller()`).
pub(crate) fn install_layered_messages(
    owner: Owner,
    messages: Value<Messages>,
    at: &'static Location<'static>,
) {
    let outer = messages_value(owner);
    // Layered in `owner`: the outer messages and this subtree's both track.
    let layered = Prop::computed(Rc::new(move || {
        outer.with(|outer| messages.with(|own| outer.layered(own)))
    }))
    .into_value(owner, at);
    install_messages(owner, layered);
}

/// The locale in effect here: the App's (`App::locale`,
/// else the system's), or the one a `kivact::view::provide_locale` above provides.
/// Reactive: it follows the App when it switches the locale at runtime,
/// and the system when the App names none and the user changes it.
///
/// Works in setup and in handlers, tasks and effects (the nearest owner).
///
/// ```
/// use kivact_core::view::text;
/// use kivact_core::i18n::use_locale;
/// use kivact_core::view::IntoView;
///
/// fn language() -> impl IntoView {
///     let locale = use_locale();
///     text(move || locale.get().language().to_owned())
/// }
/// ```
#[track_caller]
pub fn use_locale() -> Value<Locale> {
    let owner = Owner::ambient("locale", Location::caller());
    locale_value(owner)
}

/// The text of `key` in effect here: what the App set for it
/// (`App::messages`, `kivact::view::provide_messages`), else
/// `default`. Reactive: it follows the App when it switches the messages
/// at runtime. Setup only.
///
/// A component's author picks `key` as `<component>.<purpose>`
/// (`"dialog.close"`, `"pagination.page"`), keeps it stable, and writes
/// the key and its default in the component's documentation. The default
/// is English.
///
/// ```
/// use kivact_core::view::text;
/// use kivact_core::i18n::message;
/// use kivact_core::view::IntoView;
///
/// fn empty_list() -> impl IntoView {
///     text(message("list.empty", "Nothing here yet"))
/// }
/// ```
#[track_caller]
pub fn message(key: &'static str, default: &'static str) -> Value<String> {
    let owner = Owner::current("message", Location::caller());
    let messages = messages_value(owner);
    memo(move || messages.with(|m| m.get(key).unwrap_or(default).to_owned()))
}

/// Like [`message`], for a text with values in it: every `{name}` in the
/// text is replaced by the value `args` gives for `name`. `args` runs
/// again whenever what it reads changes, so the text follows the values
/// as well as the messages. A `{name}` it gives no value for stays as
/// written. A plural message uses the cardinal category of the numeric
/// `count` argument under the nearest locale. Setup only.
///
/// The App's text decides where each value goes, which is how a language
/// whose word order differs from English keeps a natural sentence
/// (`"{count} 件選択中"` for `"{count} selected"`).
///
/// ```
/// use kivact_core::view::text;
/// use kivact_core::i18n::message_with;
/// use kivact_core::view::IntoView;
/// use kivact_core::state::Signal;
///
/// fn selection(selected: Signal<Vec<u64>>) -> impl IntoView {
///     // "3 selected", or the App's wording: "3 件選択中".
///     let label = message_with("table.selected", "{count} selected", move || {
///         [("count", selected.with(Vec::len).to_string())]
///     });
///     text(label)
/// }
/// ```
#[track_caller]
pub fn message_with<A: IntoIterator<Item = (&'static str, String)>>(
    key: &'static str,
    default: &'static str,
    args: impl Fn() -> A + 'static,
) -> Value<String> {
    let owner = Owner::current("message_with", Location::caller());
    let messages = messages_value(owner);
    let locale = locale_value(owner);
    memo(move || {
        let args: Vec<(&'static str, String)> = args().into_iter().collect();
        let count = args
            .iter()
            .find(|(name, _)| *name == "count")
            .and_then(|(_, value)| value.parse::<f64>().ok());
        let category = count.map(|n| locale.with(|locale| locale.plural_category(n)));
        let template = messages.with(|messages| {
            category
                .and_then(|category| messages.get_plural(key, category))
                .or_else(|| messages.get(key))
                .unwrap_or(default)
                .to_owned()
        });
        let args: Vec<(&str, &str)> = args.iter().map(|(k, v)| (*k, v.as_str())).collect();
        interpolate(&template, &args)
    })
}
