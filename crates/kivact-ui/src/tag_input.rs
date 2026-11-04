//! Role: Render a themed field of removable tags and a text input to add one.
use super::layer::with_recipe_style;
use super::text_input::text_input;
use super::variants::{Variant, Variants};
use super::{ButtonVariant, button};
use kivact_core::custom::{NodeExt, PartState};
use kivact_core::i18n::message_with;
use kivact_core::state::signal;
use kivact_core::state::{Callback, Value};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::style::{Align, Track, auto, current_theme, style};
use kivact_core::view::NodeRef;
use kivact_core::view::{
    Handled, HandlerResult, IntoView, Key, PointerPhase, View, for_index, node_ref,
};
use kivact_core::view::{Node, NodeBuilder, grid, icon, row, text};

/// A row of removable tags and a text input to add one, created by
/// [`tag_input`].
pub struct TagInput {
    root: Node,
    label: Value<String>,
    tags: Value<Vec<String>>,
    on_add: Callback<String>,
    on_remove: Callback<usize>,
    placeholder: Option<Value<String>>,
    input_ref: NodeRef,
    variants: Variants,
}

/// A tag editor named `label` that shows `tags` and reports additions and
/// removals without writing them: the caller keeps the list.
///
/// Enter in the text input trims what was typed and, when something is left,
/// calls `on_add` with it and empties the input. Backspace in the empty input
/// and a tag's remove button (named by the message `tags.remove`, "Remove
/// {tag}") call `on_remove` with the tag's index. A press anywhere on the box
/// focuses the input. The label names the input; it is not drawn.
/// The editor is first in semantic/focus order, so a surrounding field label
/// focuses it without activating a tag's remove button. A two-column grid
/// keeps the wrapping chips before the editor visually. Root name and invalid
/// inputs are forwarded to the editor, alongside disabled state.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::tag_input;
///
/// fn labels() -> impl IntoView {
///     let tags = signal(vec!["rust".to_string()]);
///     tag_input(
///         "Labels",
///         tags,
///         move |tag| tags.update(|tags| tags.push(tag)),
///         move |index| tags.update(|tags| {
///             tags.remove(index);
///         }),
///     )
/// }
/// ```
#[track_caller]
pub fn tag_input<R: HandlerResult, S: HandlerResult>(
    label: impl IntoText,
    tags: impl IntoValue<Vec<String>>,
    on_add: impl Fn(String) -> R + 'static,
    on_remove: impl Fn(usize) -> S + 'static,
) -> TagInput {
    let variants = Variants::default();
    let input_ref = node_ref();
    // The box rings while the focus is in its input.
    let within = input_ref.focus_within();
    let root = with_recipe_style(
        grid(())
            .base_style(style().grid_template_columns((Track::minmax(0, Track::fr(1)), auto()))),
        |r| r.tag_input_recipes().root_recipe(),
        &variants,
        move || PartState {
            focused: within.get(),
            ..PartState::default()
        },
    );
    TagInput {
        root,
        label: label.into_text_value(),
        tags: tags.into_value(),
        on_add: callback(on_add),
        on_remove: callback(on_remove),
        placeholder: None,
        input_ref,
        variants,
    }
}

impl TagInput {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }

    /// The hint shown while the input is empty. The default is the message
    /// `tags.placeholder`, "Add a tag...".
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = Some(text.into_text_value());
        self
    }
}

impl NodeBuilder for TagInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TagInput {
    fn into_view(self) -> View {
        let TagInput {
            mut root,
            label,
            tags,
            on_add,
            on_remove,
            placeholder,
            input_ref,
            variants,
        } = self;
        let disabled = root
            .accessible_disabled()
            .map(|read| (move || read()).into_value())
            .unwrap_or_else(|| false.into_value());
        let label = root
            .accessible_name()
            .map(|read| (move || read()).into_value())
            .unwrap_or(label);
        let invalid = root
            .accessible_invalid()
            .map(|read| (move || read()).into_value())
            .unwrap_or_else(|| false.into_value());
        let labelled_by = root.accessible_labelled_by();
        let described_by = root.accessible_described_by();
        let pending = signal(String::new());
        let chips = for_index(tags, {
            let variants = variants.clone();
            move |index, tag: Value<String>| {
                let remove_name =
                    message_with("tags.remove", "Remove {tag}", move || [("tag", tag.get())]);
                let remove = with_recipe_style(
                    button(remove_name)
                        .disabled(disabled)
                        .variant(ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(icon("x"))
                        .on_press(move || on_remove.call(index)),
                    |r| r.tag_input_recipes().remove_recipe(),
                    &variants,
                    PartState::default,
                );
                with_recipe_style(
                    row((text(tag), remove)),
                    |r| r.tag_input_recipes().tag_recipe(),
                    &variants,
                    PartState::default,
                )
            }
        });
        let placeholder = placeholder.unwrap_or_else(|| {
            kivact_core::i18n::message("tags.placeholder", "Add a tag...").into_value()
        });
        let mut field = with_recipe_style(
            text_input(label, pending)
                .disabled(disabled)
                .invalid(invalid)
                .base_style(style().grid_row(1).grid_column(2))
                .node_ref(input_ref)
                .placeholder(placeholder)
                .on_submit(move || {
                    let tag = pending.get_untracked().trim().to_owned();
                    if !tag.is_empty() {
                        on_add.call(tag);
                        pending.set(String::new());
                    }
                })
                .on_key_default(move |event| {
                    if event.key() == Key::Backspace && pending.with_untracked(String::is_empty) {
                        if let Some(last) = tags.with_untracked(|tags| tags.len().checked_sub(1)) {
                            on_remove.call(last);
                        }
                        return Handled::Yes;
                    }
                    Handled::No
                }),
            |r| r.tag_input_recipes().input_recipe(),
            &variants,
            PartState::default,
        );
        for reference in labelled_by {
            field = field.labelled_by(reference);
        }
        for reference in described_by {
            field = field.described_by(reference);
        }
        let chips = row(chips).style(move || {
            style()
                .grid_row(1)
                .grid_column(1)
                .wrap(true)
                .min_width(0)
                .gap(current_theme().space().sm)
                .align_items(Align::Center)
        });
        root.child((field, chips))
            .on_pointer_default(move |event| {
                if matches!(event.phase(), PointerPhase::Down(_)) && !disabled.get() {
                    input_ref.focus();
                }
                Handled::No
            })
            .into_view()
    }
}
