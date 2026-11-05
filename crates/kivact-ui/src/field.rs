//! Role: Wire a visible label, hint and error to the control of a form field.
//!
//! A field is a column of the label, the control it is given, an optional
//! hint and an optional error message. The wiring is the field's: the label
//! is the control's accessible name (`labelled_by`, and the name itself),
//! the hint and the error describe it (`described_by`), and a present error
//! makes the control invalid for assistive technology and for its own
//! recipe. The error message and its relation exist only while the error is
//! `Some`. Pressing the label activates the control through the shared activation authority.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::values::FieldState;
use super::variants::{Variant, Variants};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::state::{Value, memo};
use kivact_core::view::{IntoView, Role, View, node_ref, show_some};
use kivact_core::view::{Node, NodeBuilder, node, text};

/// A control with its visible label, hint and error, created by [`field`].
///
/// The root is a column of the label, the control, the hint and the error
/// message; [`NodeBuilder`] modifiers apply to it.
// xtask-allow(state-fields): Optional label hints and validation inputs configure a field; caller-owned values determine validity.
pub struct Field<C> {
    root: Node,
    label: Value<String>,
    control: C,
    hint: Option<Value<String>>,
    required: Option<Value<bool>>,
    error: Option<Value<Option<String>>>,
    variants: Variants,
}

/// A field showing `label` over `control` (a text input, a select, a
/// switch, ...).
///
/// `label` is the visible label, and it is the control's accessible name
/// (`labelled_by`) whatever name the control was built with, so what is
/// seen is what is announced. Pressing the label activates the control as a click
/// would, focusing editable controls and toggling checks. Disabled, hidden or
/// unavailable controls stay inert through the normal activation contract. [`hint`](Field::hint) adds a
/// description under the control; [`error`](Field::error) shows a message
/// under it while the value is `Some`, marks the control invalid and adds
/// the message to what describes the control, so a validation needs no other
/// wiring.
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
///     field("Email", text_input("Email", email))
///         .hint("We never share it")
///         .error(error)
/// }
/// ```
#[track_caller]
pub fn field<C: NodeBuilder + IntoView>(label: impl IntoText, control: C) -> Field<C> {
    Field {
        root: node(),
        label: label.into_text_value(),
        control,
        hint: None,
        required: None,
        error: None,
        variants: Variants::default(),
    }
}

impl<C> Field<C> {
    /// Text under the control that describes it (`described_by`).
    pub fn hint(mut self, hint: impl IntoText) -> Self {
        self.hint = Some(hint.into_text_value());
        self
    }

    /// Report that the control requires a value. Validation is owned by the app.
    pub fn required(mut self, required: impl IntoValue<bool>) -> Self {
        self.required = Some(required.into_value());
        self
    }

    /// The error message under the hint while the value is `Some`. While it
    /// is, the control is invalid (its own `invalid` value is kept: the
    /// control is invalid when either is), the label is drawn as an error
    /// and the message, a `Role::Alert`, describes the control.
    pub fn error(mut self, error: impl IntoValue<Option<String>>) -> Self {
        self.error = Some(error.into_value());
        self
    }

    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}

impl<C> NodeBuilder for Field<C> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[diagnostic::do_not_recommend]
impl<C: NodeBuilder + IntoView> IntoView for Field<C> {
    fn into_view(self) -> View {
        let Field {
            root,
            label,
            mut control,
            hint,
            required,
            error,
            variants,
        } = self;
        let (label_ref, hint_ref, error_ref, control_ref) =
            (node_ref(), node_ref(), node_ref(), node_ref());
        control = control
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
            let own = control.node_mut().accessible_invalid();
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
                |recipes| recipes.field_recipes().label_recipe(),
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
                    |recipes| recipes.field_recipes().hint_recipe(),
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
                        |recipes| recipes.field_recipes().error_recipe(),
                        &variants,
                        move || state.get(),
                    )
                })
                .into_view(),
            );
        }
        with_base_recipe_style(
            root,
            |recipes| recipes.field_recipes().root_recipe(),
            &variants,
            move || state.get(),
        )
        .child(parts)
        .into_view()
    }
}
