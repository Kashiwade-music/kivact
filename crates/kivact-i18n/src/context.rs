//! Role: This module reads locale contexts and reactive messages.

use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::{Owner, Prop, Value, memo};

use super::locale::Locale;
use super::messages::Messages;

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
pub fn install_locale(owner: Owner, locale: Value<Locale>) {
    owner.provide(LocaleCtx(locale));
}

/// Provide `messages` to `owner`'s subtree, replacing the inherited ones
/// (the App's root uses it).
pub fn install_messages(owner: Owner, messages: Value<Messages>) {
    owner.provide(MessagesCtx(messages));
}

/// Provide `messages` to `owner`'s subtree layered over the messages
/// `owner` inherits: a key found in `messages` wins, others fall back.
/// Both sides stay reactive: a switch on either reaches the readers.
///
/// `at` is the creation site recorded for the layered value (pass the
/// caller's `Location::caller()`).
pub fn install_layered_messages(
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
/// use kivact::i18n::locale;
/// use kivact::prelude::*;
///
/// fn language() -> impl IntoView {
///     let locale = locale();
///     text(move || locale.get().language().to_owned())
/// }
/// ```
#[track_caller]
pub fn locale() -> Value<Locale> {
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
/// use kivact::i18n::message;
/// use kivact::prelude::*;
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
/// written. Setup only.
///
/// The App's text decides where each value goes, which is how a language
/// whose word order differs from English keeps a natural sentence
/// (`"{count} 件選択中"` for `"{count} selected"`).
///
/// ```
/// use kivact::i18n::message_with;
/// use kivact::prelude::*;
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
    memo(move || {
        let template = messages.with(|m| m.get(key).unwrap_or(default).to_owned());
        let args: Vec<(&'static str, String)> = args().into_iter().collect();
        let args: Vec<(&str, &str)> = args.iter().map(|(k, v)| (*k, v.as_str())).collect();
        interpolate(&template, &args)
    })
}

/// `template` with each `{name}` replaced by the value `args` holds for
/// `name` (what [`message_with`] does on each change). For code that builds
/// a text where no setup is available, such as a drawing closure: read the
/// template once in setup with [`message`] (a drawing closure may read the
/// returned value, and runs again when it changes), then fill it where the
/// values are known. A `{name}` that `args` does not name stays as written.
///
/// ```
/// use kivact::i18n::interpolate;
///
/// assert_eq!(
///     interpolate("Link {from} to {to}", &[("from", "A"), ("to", "B")]),
///     "Link A to B"
/// );
/// assert_eq!(interpolate("{x} and {y}", &[("x", "1")]), "1 and {y}");
/// ```
pub fn interpolate(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let tail = &rest[open + 1..];
        match tail.find('}') {
            Some(close) => {
                let name = &tail[..close];
                match args.iter().find(|(key, _)| *key == name) {
                    Some((_, value)) => out.push_str(value),
                    None => out.push_str(&rest[open..open + close + 2]),
                }
                rest = &tail[close + 1..];
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}
