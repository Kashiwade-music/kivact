//! Role: The tag input builder, its tag chips and the add field.

use crate::helpers::{is_named, report_unnamed};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::tag_input::look::TagInputTheme;
use crate::text_input::{TextInput, text_input};
use crate::{ButtonVariant, button};
use kivact_core::custom::PartState;
use kivact_core::i18n::message_with;
use kivact_core::state::signal;
use kivact_core::state::{Callback, Signal, Value};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::style::{Align, Track, auto, style, use_theme};
use kivact_core::view::NodeRef;
use kivact_core::view::Variants;
use kivact_core::view::{Element, Node, grid, icon, row, text};
use kivact_core::view::{
    Handled, HandlerResult, IntoView, Key, PointerPhase, View, for_index, node_ref,
};
use std::panic::Location;

/// A row of removable tags and a text input to add one, created by
/// [`tag_input`].
pub struct TagInput {
    root: Node,
    tags: Signal<Vec<String>>,
    on_edit: Option<Callback<kivact_core::view::EditPhase>>,
    placeholder: Value<String>,
    input_ref: NodeRef,
    at: &'static Location<'static>,
}

/// A tag editor that edits the list `tags`. Enter in the text input trims
/// what was typed and, when something is left, appends it to the list and
/// empties the input. Backspace in the empty input and a tag's remove button
/// (named by the message `tags.remove`, "Remove {tag}") remove the last tag or
/// that tag. A press anywhere on the box focuses the input. Name the input
/// with [`label`](TagInput::label); it is not drawn.
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
///     tag_input(tags).label("Labels")
/// }
/// ```
#[track_caller]
pub fn tag_input(tags: Signal<Vec<String>>) -> TagInput {
    let input_ref = node_ref();
    let root = grid(())
        .base_style(style().grid_template_columns((Track::minmax(0, Track::fr(1)), auto())));
    TagInput {
        root,
        tags,
        on_edit: None,
        placeholder: kivact_core::i18n::message("tags.placeholder", "Add a tag..."),
        input_ref,
        at: Location::caller(),
    }
}

impl TagInput {
    /// The editor's accessible name (the root's name, as `.name` sets it).
    /// Inside a [`field`](crate::field) the field's label names it.
    #[track_caller]
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.root = self.root.name(label);
        self
    }

    /// Observe each addition or removal as one complete edit.
    pub fn on_edit<R: HandlerResult>(
        mut self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(callback(f));
        self
    }

    /// The hint shown while the input is empty. The default is the message
    /// `tags.placeholder`, "Add a tag...".
    #[track_caller]
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = text.into_text_value();
        self
    }
}

impl Element for TagInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TagInput {
    fn into_view(self) -> View {
        let TagInput {
            mut root,
            tags,
            on_edit,
            placeholder,
            input_ref,
            at,
        } = self;
        let variants = root.authored().variants();
        let disabled = root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        report_unnamed(is_named(&mut root), "tag_input", at);
        let label = root
            .accessible_name()
            .unwrap_or_else(|| String::new().into_value());
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
        let chips = tag_chips(tags, disabled, edit, &variants);
        let field = with_recipe_style(
            text_input(pending)
                .label(label)
                .disabled(disabled)
                .invalid(invalid)
                .with_node(|node| node.base_style(style().grid_row(1).grid_column(2)))
                .node_ref(input_ref)
                .placeholder(placeholder)
                .on_submit(move || {
                    let tag = pending.get_untracked().trim().to_owned();
                    if !tag.is_empty() {
                        edit.once((), || tags.update(|tags| tags.push(tag)));
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
                                edit.once((), || remove_tag(tags, last));
                            }
                            return Handled::Yes;
                        }
                        Handled::No
                    })
                }),
            |r: &TagInputTheme| r.input_recipe(),
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
        |r: &TagInputTheme| r.root_recipe(),
        variants,
        move || PartState {
            focus: if within.get() {
                kivact_core::view::Focus::Focused
            } else {
                kivact_core::view::Focus::None
            },
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
/// Remove the tag at `index` from the list, if there is one.
fn remove_tag(tags: Signal<Vec<String>>, index: usize) {
    tags.update(|tags| {
        if index < tags.len() {
            tags.remove(index);
        }
    });
}

fn tag_chips(
    tags: Signal<Vec<String>>,
    disabled: Value<bool>,
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
                    .on_press(move || edit.once((), || remove_tag(tags, index))),
                |r: &TagInputTheme| r.remove_recipe(),
                &variants,
                PartState::default,
            );
            with_recipe_style(
                row((text(tag), remove)),
                |r: &TagInputTheme| r.tag_recipe(),
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
            .flex_wrap(true)
            .min_width(0)
            .gap(use_theme().get().get(&kivact_core::style::space::SM_VAR))
            .align_items(Align::Center)
    })
}
