//! Role: The tag input component, its tag chips and the add field.

use crate::helpers::report_unnamed;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::tag_input::look::TagInputTheme;
use crate::text_input::{TextInput, text_input};
use crate::{ButtonVariant, button};
use kivact_core::custom::{Caller, PartFacts, PartState};
use kivact_core::i18n::{message, message_with};
use kivact_core::state::signal;
use kivact_core::state::{Callback, IntoValue, Signal, Value};
use kivact_core::style::{Align, Track, auto, style, use_theme};
use kivact_core::view::NodeRef;
use kivact_core::view::Variants;
use kivact_core::view::{
    Element, Handled, IntoView, Key, Node, PointerPhase, for_index, grid, icon, node_ref, row, text,
};
use kivact_macros::component;

/// A tag editor that edits the list `tags`. Enter in the text input trims
/// what was typed and, when something is left, appends it to the list and
/// empties the input. Backspace in the empty input and a tag's remove button
/// (named by the message `tags.remove`, "Remove {tag}") remove the last tag or
/// that tag. A press anywhere on the box focuses the input. Name the input
/// with `label`; it is not drawn.
/// The editor is first in semantic/focus order, so a surrounding field label
/// focuses it without activating a tag's remove button. A two-column grid
/// keeps the wrapping chips before the editor visually. Root name and invalid
/// inputs are forwarded to the editor, alongside disabled state.
///
/// Setters: `label` names the input; `on_edit` observes each addition or
/// removal as one complete edit; `placeholder` is the hint shown while the
/// input is empty (default: the message `tags.placeholder`, "Add a tag...").
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
#[component]
pub fn tag_input(
    tags: Signal<Vec<String>>,
    label: Option<Value<String>>,
    on_edit: Option<Callback<kivact_core::view::EditPhase>>,
    placeholder: Option<Value<String>>,
    #[caller] caller: Caller,
) -> impl Element {
    let named = label.is_some() || caller.name().is_some() || !caller.labelled_by().is_empty();
    report_unnamed(named, "tag_input", caller.at());
    let variants = caller.variants();
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let invalid = caller.invalid().unwrap_or_else(|| false.into_value());
    let labelled_by = caller.labelled_by().to_vec();
    let described_by = caller.described_by().to_vec();
    let placeholder = placeholder.unwrap_or_else(|| message("tags.placeholder", "Add a tag..."));
    let root = grid(())
        .base_style(style().grid_template_columns((Track::minmax(0, Track::fr(1)), auto())));
    let root = match label {
        Some(label) => root.name(label),
        None => root,
    };
    // The editor's name: the label, or the caller's name. Without one the
    // editor is named "" and the missing name is reported.
    let editor = label
        .or_else(|| caller.name())
        .unwrap_or_else(|| String::new().into_value());
    let input_ref = node_ref();
    let edit = kivact_interaction::EditSession::new(|(): ()| {});
    if let Some(observer) = on_edit {
        edit.on_edit(move |phase| observer.call(phase));
    }
    let pending = signal(String::new());
    let chips = tag_chips(tags, disabled, edit, &variants);
    let field = with_recipe_style(
        text_input(pending)
            .label(editor)
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
                    if event.key() == Key::Backspace && pending.with_untracked(String::is_empty) {
                        if let Some(last) = tags.with_untracked(|tags| tags.len().checked_sub(1)) {
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
) -> Node {
    let chips = chip_row(chips);
    // The box rings while the focus is in its input.
    let within = input_ref.focus_within();
    with_base_recipe_style(
        root,
        |r: &TagInputTheme| r.root_recipe(),
        variants,
        move || {
            PartState::fixed(PartFacts {
                focus: if within.get() {
                    kivact_core::view::Focus::Focused
                } else {
                    kivact_core::view::Focus::None
                },
                ..PartFacts::default()
            })
        },
    )
    .child((field, chips))
    .on_pointer_default(move |event| {
        if matches!(event.phase(), PointerPhase::Down(_)) && !disabled.get() {
            input_ref.focus();
        }
        Handled::No
    })
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
