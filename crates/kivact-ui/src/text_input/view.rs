//! Role: The text input and text area builders and their editing state.

use crate::text_input::look::TextAreaTheme;
use crate::text_input::look::TextInputTheme;
use kivact_core::style::use_theme;

use kivact_core::custom::Caller;
use kivact_core::state::{Callback, IntoValue, Signal, Value};
use kivact_core::view::{Composition, EditPhase, InputPurpose, Key, NodeRef, node};
use kivact_interaction::{use_text_area, use_text_edit};
use kivact_macros::component;

use crate::control_size::{ControlSize, default_control_size};
use crate::helpers::report_unnamed;
use crate::text_input::look::TextInputState;

/// A single-line text input with the theme's field look.
///
/// It edits `value`: the text shown is the signal's, and each committed
/// edit (never an IME composition in progress) is written to it. Changes
/// are observed through the signal: to run more code on each edit, pass
/// `binding(read, write)` with that code in `write`. Name it with
/// `.label(..)`, or with [`field`](crate::field).
///
/// The look is the theme's [`TextInputTheme::root`](crate::theme::TextInputTheme):
/// a bordered, rounded field that draws a focus ring. Text is edited
/// natively (caret, selection, IME, clipboard), with `Role::TextBox`. With
/// `multiline(true)` the field is a text area (see [`text_area`]).
/// [`Element`] modifiers apply to the native field.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn search() -> impl IntoView {
///     let query = signal(String::new());
///     column((
///         text_input(query).label("Search").placeholder("Type to search"),
///         text(move || format!("Searching for {}", query.get())),
///     ))
///     .style(style().gap(8))
/// }
/// ```
///
/// Setters: `wrap` and `rows` (text areas), `forward_keys` (keys the handlers
/// get), `size` (the metrics, default medium), `placeholder`, `purpose` (the
/// keyboard), `read_only`, `max_length`, `on_submit` (Enter), `on_edit` (the
/// [`EditPhase`]s), `return_focus` (where the focus goes when the draft ends)
/// and `on_composition_change`. `multiline` (default false) makes a text area.
///
/// `on_commit` edits the text as a draft, which the user commits or
/// abandons. The binding the field was made with holds the text as it is
/// typed; `on_commit` receives it when the user presses Enter or leaves the
/// field.
///
/// - Enter commits: `on_commit` runs with the text, and the draft ends
///   when it returns `()` or `Ok`. An `Err` is reported and the draft stays
///   open. A field marked [`invalid`](Element::invalid) does not commit; it
///   stays as it is.
/// - Escape cancels: the text from before the draft is written back to the
///   binding.
/// - Leaving the field ends the draft the same way: a text that is
///   unchanged is cancelled, a changed and valid one is committed, and an
///   invalid one stays pending.
///
/// The draft is an [`EditSession`](kivact_interaction::EditSession): it
/// begins when the field gets the focus or when the user edits it, and
/// `on_edit` reports its [`EditPhase`]s, so a field that is shown only while
/// editing (a rename) closes itself on `Commit` and `Cancel`. `return_focus`
/// says where the focus goes when Enter or Escape ends it. Enter is the
/// draft's, so this replaces `on_submit`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::view::EditPhase;
///
/// fn rename(name: Signal<String>) -> impl IntoView {
///     let draft = signal(name.get_untracked());
///     let editing = signal(true);
///     show(editing, move || {
///         text_input(draft).label("Name")
///             .on_commit(move |text| name.set(text))
///             .on_edit(move |phase| {
///                 if phase != EditPhase::Begin {
///                     editing.set(false);
///                 }
///             })
///     })
/// }
/// ```
#[component]
pub fn text_input(
    value: Signal<String>,
    #[prop(default = false)] multiline: bool,
    label: Option<Value<String>>,
    wrap: Option<Value<bool>>,
    rows: Option<std::ops::RangeInclusive<usize>>,
    forward_keys: Option<Value<Vec<Key>>>,
    size: Option<Value<ControlSize>>,
    placeholder: Option<Value<String>>,
    purpose: Option<Value<InputPurpose>>,
    read_only: Option<Value<bool>>,
    max_length: Option<Value<usize>>,
    on_submit: Option<Callback<()>>,
    on_commit: Option<Callback<String>>,
    on_edit: Option<Callback<EditPhase>>,
    return_focus: Option<NodeRef>,
    on_composition_change: Option<Callback<Composition>>,
    #[caller] caller: Caller,
) -> impl Element {
    let named = label.is_some() || caller.name().is_some() || !caller.labelled_by().is_empty();
    let what = if multiline { "text_area" } else { "text_input" };
    report_unnamed(named, what, caller.at());
    let invalid = caller.invalid().unwrap_or_else(|| false.into_value());
    let look = caller.variants();
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let grows = multiline && rows.is_some();
    let mut editing = if multiline {
        use_text_area(value)
    } else {
        use_text_edit(value)
    };
    if let Some(on) = wrap {
        editing = editing.wrap(on);
    }
    if let Some(rows) = rows {
        editing = editing.rows(rows);
    }
    if let Some(keys) = forward_keys {
        editing = editing.forward_keys(keys);
    }
    if let Some(placeholder) = placeholder {
        editing = editing.placeholder(placeholder);
    }
    if let Some(purpose) = purpose {
        editing = editing.purpose(purpose);
    }
    if let Some(read_only) = read_only {
        editing = editing.read_only(read_only);
    }
    if let Some(max) = max_length {
        editing = editing.max_length(max);
    }
    if let Some(submit) = on_submit {
        editing = editing.on_submit(submit);
    }
    if let Some(commit) = on_commit {
        editing = editing.on_commit(move |text: String| commit.invoke(text));
    }
    if let Some(edit) = on_edit {
        editing = editing.on_edit(move |phase: EditPhase| edit.invoke(phase));
    }
    if let Some(target) = return_focus {
        editing = editing.return_focus(target);
    }
    if let Some(changed) = on_composition_change {
        editing = editing.on_composition_change(move |state: Composition| changed.invoke(state));
    }
    let part = editing.control();
    let input = part.state();
    let mut field = node().part(part);
    if let Some(label) = label {
        field = field.name(label);
    }
    if grows {
        field = field.style(kivact_core::style::style().height(kivact_core::style::auto()));
    }
    field.base_style(move || {
        let theme = use_theme().get();
        let state = crate::recipe::Look::of(
            TextInputState {
                input: input.get(),
                size: size.get(),
                invalid: invalid.get(),
            },
            look.clone(),
        );
        if multiline {
            theme
                .ext::<TextAreaTheme>()
                .root_recipe()
                .style(&theme, &state)
        } else {
            theme
                .ext::<TextInputTheme>()
                .root_recipe()
                .style(&theme, &state)
        }
    })
}

/// A controlled native multiline input with the theme's text-area recipe:
/// [`text_input`] with `multiline(true)`.
#[track_caller]
pub fn text_area(value: Signal<String>) -> TextInput {
    text_input(value).multiline(true)
}
