//! Role: The field component that wires a label, hint and error to its control.

use crate::field::look::FieldState;
use crate::field::look::FieldTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::Caller;
use kivact_core::state::{Value, memo};
use kivact_core::view::{Element, IntoView, Role, View, node, node_ref, show_some, text};
use kivact_macros::component;

/// A field showing `label` over `control` (a text input, a select, a
/// switch, ...). Its root is a column of the label, the control, the hint
/// and the error message; [`Element`] modifiers apply to it.
///
/// `label` is the visible label, and it is the control's accessible name
/// (`labelled_by`) whatever name the control was built with, so what is
/// seen is what is announced. Pressing the label activates the control as a click
/// would, focusing editable controls and toggling checks. Disabled, hidden or
/// unavailable controls stay inert through the normal activation contract.
///
/// Setters: `hint` adds a description under the control; `required` reports
/// that the control requires a value (validation is owned by the app); and
/// `error` shows a message under the control while the value is `Some`,
/// marks the control invalid and adds the message to what describes the
/// control, so a validation needs no other wiring. The control's own
/// `invalid` value is kept: the control is invalid when either is. While an
/// error is shown, the label is drawn as an error and the message, a
/// `Role::Alert`, describes the control.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn email() -> impl IntoView {
///     let email = signal(String::new());
///     let error = memo(move || {
///         let email = email.get();
///         (!email.is_empty() && !email.contains('@')).then(|| "Enter an email address".to_owned())
///     });
///     field("Email", text_input(email).label("Email"))
///         .hint("We never share it")
///         .error(error)
/// }
/// ```
#[component]
pub fn field<C: Element + IntoView>(
    label: Value<String>,
    control: C,
    hint: Option<Value<String>>,
    required: Option<Value<bool>>,
    error: Option<Value<Option<String>>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let (label_ref, hint_ref, error_ref, control_ref) =
        (node_ref(), node_ref(), node_ref(), node_ref());
    let mut control = control
        .name(label)
        .with_node(|node| node.labelled_by(label_ref))
        .node_ref(control_ref);
    if let Some(required) = required {
        control = control.with_node(|node| node.required(required));
    }
    if hint.is_some() {
        control = control.with_node(|node| node.described_by(hint_ref));
    }
    let failed = memo(move || error.is_some_and(|error| error.with(Option::is_some)));
    if error.is_some() {
        let own = control.node_mut().authored().invalid();
        control = control
            .with_node(|node| node.described_by(error_ref))
            .invalid(move || failed.get() || own.as_ref().is_some_and(|own| own.get()));
    }
    let state = memo(move || FieldState {
        invalid: failed.get(),
    });
    let mut parts: Vec<View> = vec![
        with_recipe_style(
            text(label)
                .role(Role::Text)
                .node_ref(label_ref)
                .on_press(move || control_ref.activate()),
            |r: &FieldTheme| r.label_recipe(),
            &variants,
            move || state.get(),
        )
        .into_view(),
        control.into_view(),
    ];
    if let Some(hint) = hint {
        parts.push(
            with_recipe_style(
                text(hint).node_ref(hint_ref),
                |r: &FieldTheme| r.hint_recipe(),
                &variants,
                move || state.get(),
            )
            .into_view(),
        );
    }
    if let Some(error) = error {
        let variants = variants.clone();
        parts.push(
            show_some(error, move |message| {
                with_recipe_style(
                    text(message).role(Role::Alert).node_ref(error_ref),
                    |r: &FieldTheme| r.error_recipe(),
                    &variants,
                    move || state.get(),
                )
            })
            .into_view(),
        );
    }
    with_base_recipe_style(
        node(),
        |r: &FieldTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    )
    .child(parts)
}
