//! Role: Render a themed field of removable tags and a text input to add one.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::text_input::{TextInput, text_input};
use super::variants::{Variant, Variants};
use super::{ButtonVariant, button};
use kivact_core::custom::PartState;
use kivact_core::i18n::message_with;
use kivact_core::state::signal;
use kivact_core::state::{Callback, Value};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::style::{Align, Track, auto, style, use_theme};
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
    on_edit: Option<Callback<kivact_core::view::EditPhase>>,
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
///     tag_input("Labels", tags)
///         .on_add(move |tag| tags.update(|tags| tags.push(tag)))
///         .on_remove(move |index| tags.update(|tags| {
///             tags.remove(index);
///         }))
/// }
/// ```
#[track_caller]
pub fn tag_input(label: impl IntoText, tags: impl IntoValue<Vec<String>>) -> TagInput {
    let input_ref = node_ref();
    let root = grid(())
        .base_style(style().grid_template_columns((Track::minmax(0, Track::fr(1)), auto())));
    TagInput {
        root,
        label: label.into_text_value(),
        tags: tags.into_value(),
        on_edit: None,
        on_add: callback(|_: String| {}),
        on_remove: callback(|_: usize| {}),
        placeholder: None,
        input_ref,
        variants: Variants::default(),
    }
}

impl TagInput {
    /// Observe each addition or removal as one complete edit.
    pub fn on_edit<R: HandlerResult>(
        mut self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(callback(f));
        self
    }

    /// Handle a new tag.
    pub fn on_add<R: HandlerResult>(mut self, f: impl Fn(String) -> R + 'static) -> Self {
        self.on_add = callback(f);
        self
    }
    /// Handle removal of a tag by index.
    pub fn on_remove<R: HandlerResult>(mut self, f: impl Fn(usize) -> R + 'static) -> Self {
        self.on_remove = callback(f);
        self
    }
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards.
    pub fn variant(mut self, v: impl Variant) -> Self {
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
            on_edit,
            on_add,
            on_remove,
            placeholder,
            input_ref,
            variants,
        } = self;
        let disabled = root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let label = root.accessible_name().unwrap_or(label);
        let invalid = root
            .accessible_invalid()
            .unwrap_or_else(|| false.into_value());
        let labelled_by = root.accessible_labelled_by();
        let described_by = root.accessible_described_by();
        let edit = kivact_interaction::EditSession::new(|(): ()| {});
        if let Some(observer) = on_edit {
            edit.on_edit(move |phase| observer.call(phase));
        }
        let pending = signal(String::new());
        let chips = tag_chips(tags, disabled, on_remove, edit, &variants);
        let placeholder = placeholder.unwrap_or_else(|| {
            kivact_core::i18n::message("tags.placeholder", "Add a tag...").into_value()
        });
        let field = with_recipe_style(
            text_input(label, pending)
                .disabled(disabled)
                .invalid(invalid)
                .with_node(|node| node.base_style(style().grid_row(1).grid_column(2)))
                .node_ref(input_ref)
                .placeholder(placeholder)
                .on_submit(move || {
                    let tag = pending.get_untracked().trim().to_owned();
                    if !tag.is_empty() {
                        edit.once((), || on_add.call(tag));
                        pending.set(String::new());
                    }
                })
                .with_node(|node| {
                    node.on_key_default(move |event| {
                        if event.key() == Key::Backspace && pending.with_untracked(String::is_empty)
                        {
                            if let Some(last) =
                                tags.with_untracked(|tags| tags.len().checked_sub(1))
                            {
                                edit.once((), || on_remove.call(last));
                            }
                            return Handled::Yes;
                        }
                        Handled::No
                    })
                }),
            |r| r.tag_input_recipes().input_recipe(),
            &variants,
            PartState::default,
        );
        let field = editor_references(field, labelled_by, described_by);
        with_tag_box(root, field, chips, &variants, disabled, input_ref)
    }
}

/// The editor with the caller's accessibility references.
fn editor_references(
    mut field: TextInput,
    labelled_by: Vec<NodeRef>,
    described_by: Vec<NodeRef>,
) -> TextInput {
    for reference in labelled_by {
        field = field.with_node(|node| node.labelled_by(reference));
    }
    for reference in described_by {
        field = field.with_node(|node| node.described_by(reference));
    }
    field
}

/// The finished box: the editor and its chips, ringing while the editor has focus.
fn with_tag_box(
    root: Node,
    field: TextInput,
    chips: impl IntoView,
    variants: &Variants,
    disabled: Value<bool>,
    input_ref: NodeRef,
) -> View {
    let chips = chip_row(chips);
    // The box rings while the focus is in its input.
    let within = input_ref.focus_within();
    with_base_recipe_style(
        root,
        |r| r.tag_input_recipes().root_recipe(),
        variants,
        move || PartState {
            focused: within.get(),
            ..PartState::default()
        },
    )
    .child((field, chips))
    .on_pointer_default(move |event| {
        if matches!(event.phase(), PointerPhase::Down(_)) && !disabled.get() {
            input_ref.focus();
        }
        Handled::No
    })
    .into_view()
}

/// Render application-owned tags and report each removal as one edit.
fn tag_chips(
    tags: Value<Vec<String>>,
    disabled: Value<bool>,
    on_remove: Callback<usize>,
    edit: kivact_interaction::EditSession<()>,
    variants: &Variants,
) -> impl IntoView {
    for_index(tags, {
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
                    .on_press(move || edit.once((), || on_remove.call(index))),
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
    })
}

/// Lay out wrapping chips before the editor in the input grid.
fn chip_row(chips: impl IntoView) -> impl IntoView {
    row(chips).style(move || {
        style()
            .grid_row(1)
            .grid_column(1)
            .wrap(true)
            .min_width(0)
            .gap(use_theme().get().space().sm)
            .align_items(Align::Center)
    })
}
