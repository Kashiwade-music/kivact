//! Role: `text_input`.
//!
//! `text_input` and `text_area` put the
//! [`TextEditApi`] control on an ordinary node and
//! hand the native settings and callbacks to the `TextEditApi`; the field's
//! style is the theme's `text_input` or `text_area` recipe with a
//! [`TextInputState`]. [`TextInput`] implements `NodeBuilder` with the field
//! as its node, so modifiers land on the input itself. `Style::paint` is box
//! drawing, which a native editing leaf does not support, so these recipes
//! return style only.

use kivact_core::style::use_theme;
use std::cell::{Cell, OnceCell};
use std::rc::Rc;

use kivact_core::state::Binding;
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::view::EditPhase;
use kivact_core::view::{HandlerResult, IntoView, NodeRef, View};
use kivact_core::view::{Node, NodeBuilder, node};
use kivact_interaction::{TextEditApi, use_text_area, use_text_edit};

use super::layer::recipes;
use super::values::{ControlSize, TextInputState};
use super::variants::{Variant, Variants};

/// A single-line text input with the theme's field look.
///
/// Created by [`text_input`]. Native editing, binding and IME use
/// [`TextEditApi`](kivact_interaction::TextEditApi). [`NodeBuilder`] modifiers
/// apply to the native field.
pub struct TextInput {
    field: Node,
    editing: TextEditApi,
    size: Rc<Cell<kivact_core::state::Value<ControlSize>>>,
    invalid: Rc<OnceCell<kivact_core::state::Value<bool>>>,
    variants: Variants,
    multiline: bool,
}

/// A text input named `label` (its accessible name; a later `.name(..)`
/// replaces it) that edits `value`: the text shown is the binding's, and
/// each committed edit (never an IME composition in progress) is written
/// to it. Changes are observed through the binding: to run more code on
/// each edit, pass `binding(read, write)` with that code in `write`.
///
/// The look is the theme's [`UiRecipes::text_input`](super::UiRecipes):
/// a bordered, rounded field that draws a focus ring. Text is edited
/// natively (caret, selection, IME, clipboard), with `Role::TextBox`.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn search() -> impl IntoView {
///     let query = signal(String::new());
///     column((
///         text_input("Search", query).placeholder("Type to search"),
///         text(move || format!("Searching for {}", query.get())),
///     ))
///     .style(style().gap(8))
/// }
/// ```
#[track_caller]
pub fn text_input(
    label: impl IntoText,
    value: impl kivact_core::state::IntoBinding<String>,
) -> TextInput {
    create(label.into_text_value(), value.into_binding(), false)
}
/// A controlled native multiline input with the theme's text-area recipe.
pub fn text_area(
    label: impl IntoText,
    value: impl kivact_core::state::IntoBinding<String>,
) -> TextInput {
    create(label.into_text_value(), value.into_binding(), true)
}
fn create(
    label: kivact_core::state::Value<String>,
    value: Binding<String>,
    multiline: bool,
) -> TextInput {
    let size = Rc::new(Cell::new(
        super::values::default_control_size().into_value(),
    ));
    let read = size.clone();
    let invalid = Rc::new(OnceCell::<kivact_core::state::Value<bool>>::new());
    let read_invalid = invalid.clone();
    let editing = if multiline {
        use_text_area(value)
    } else {
        use_text_edit(value)
    };
    let part = editing.control();
    let input = part.state();
    let variants = Variants::default();
    let look = variants.clone();
    let field = node().name(label).part(part).style(move || {
        let theme = use_theme().get();
        let recipes = recipes(&theme);
        let recipe = if multiline {
            recipes.text_area_recipe()
        } else {
            recipes.text_input_recipe()
        };
        recipe(
            &theme,
            &look.look(TextInputState {
                input: input.get(),
                size: read.get().get(),
                invalid: read_invalid.get().is_some_and(|invalid| invalid.get()),
            }),
        )
    });
    TextInput {
        field,
        editing,
        size,
        invalid,
        variants,
        multiline,
    }
}

impl TextInput {
    /// Soft-wrap a text area's visual rows at its content width.
    pub fn wrap(mut self, on: impl IntoValue<bool>) -> Self {
        self.editing = self.editing.wrap(on);
        self
    }

    /// Grow a text area between these inclusive visual row counts.
    pub fn rows(mut self, rows: std::ops::RangeInclusive<usize>) -> Self {
        self.editing = self.editing.rows(rows);
        if self.multiline {
            self.field = self
                .field
                .style(kivact_core::style::style().height(kivact_core::style::auto()));
        }
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

impl TextInput {
    /// Route selected control keys to handlers, for example Tab between picker columns.
    ///
    /// Tab, arrows, Home, End, PageUp, PageDown, Escape and Enter are supported;
    /// characters and Space are ignored. An editing key native editing has
    /// nothing to do with (Backspace or Delete in an empty input) already
    /// reaches the key handlers without being forwarded. Ctrl/Alt/Meta and Shift with keys other
    /// than Tab retain native editing. IME composition consumes keys first.
    /// The default forwards nothing. Once forwarded, a key has no native fallback
    /// even when its handler returns `Handled::No`. The list may be reactive;
    /// configure the source before mounting, and its later changes still
    /// apply after the mount. The rules are
    /// [`TextEditApi::forward_keys`](kivact_interaction::TextEditApi::forward_keys)'s.
    #[track_caller]
    pub fn forward_keys(mut self, keys: impl IntoValue<Vec<kivact_core::view::Key>>) -> Self {
        self.editing = self.editing.forward_keys(keys);
        self
    }

    /// The metrics (default [`ControlSize::Medium`]).
    pub fn size(self, size: impl IntoValue<ControlSize>) -> Self {
        self.size.set(size.into_value());
        self
    }

    /// Hint text shown while the input is empty.
    pub fn placeholder(mut self, placeholder: impl IntoText) -> Self {
        self.editing = self.editing.placeholder(placeholder);
        self
    }

    /// Show the text as dots, constant or reactive.
    pub fn password(mut self, password: impl IntoValue<bool>) -> Self {
        self.editing = self.editing.password(password);
        self
    }

    /// Allow selecting and copying but not editing, constant or reactive.
    pub fn read_only(mut self, read_only: impl IntoValue<bool>) -> Self {
        self.editing = self.editing.read_only(read_only);
        self
    }

    /// The longest text the input takes, in characters.
    pub fn max_length(mut self, max: impl IntoValue<usize>) -> Self {
        self.editing = self.editing.max_length(max);
        self
    }

    /// Called when Enter is pressed.
    pub fn on_submit(mut self, f: impl Into<kivact_core::state::Callback<()>>) -> Self {
        self.editing = self.editing.on_submit(f);
        self
    }

    /// Edit the text as a draft, which the user commits or abandons. The
    /// binding the field was made with holds the text as it is typed; `on_commit`
    /// receives it when the user presses Enter or leaves the field.
    ///
    /// - Enter commits: `on_commit` runs with the text, and the draft ends
    ///   when it returns `()` or `Ok`. An `Err` is reported and the draft stays
    ///   open. A field marked [`invalid`](NodeBuilder::invalid) does not
    ///   commit; it stays as it is.
    /// - Escape cancels: the text from before the draft is written back to
    ///   the binding.
    /// - Leaving the field ends the draft the same way: a text that is
    ///   unchanged is cancelled, a changed and valid one is committed, and an
    ///   invalid one stays pending.
    ///
    /// The draft is an [`EditSession`](kivact_interaction::EditSession): it
    /// begins when the field gets the focus or when the user edits it, and
    /// [`on_edit`](Self::on_edit) reports its [`EditPhase`]s, so a field that
    /// is shown only while editing (a rename) closes itself on `Commit` and
    /// `Cancel`. [`return_focus`](Self::return_focus) says where the focus
    /// goes when Enter or Escape ends it. Enter is the draft's, so this
    /// replaces `on_submit`. Setup only.
    ///
    /// ```no_run
    /// use kivact::prelude::*;
    /// use kivact::view::EditPhase;
    ///
    /// fn rename(name: Signal<String>) -> impl IntoView {
    ///     let draft = signal(name.get_untracked());
    ///     let editing = signal(true);
    ///     show(editing, move || {
    ///         text_input("Name", draft)
    ///             .on_draft_commit(move |text| name.set(text))
    ///             .on_edit(move |phase| {
    ///                 if phase != EditPhase::Begin {
    ///                     editing.set(false);
    ///                 }
    ///             })
    ///     })
    /// }
    /// ```
    #[track_caller]
    pub fn on_draft_commit<R: HandlerResult>(
        mut self,
        on_commit: impl Fn(String) -> R + 'static,
    ) -> Self {
        self.editing = self.editing.on_draft_commit(on_commit);
        self
    }

    /// Report each [`EditPhase`] of the draft ([`draft`](Self::on_draft_commit)):
    /// `Begin` when it starts, then `Commit` or `Cancel`. Without a draft,
    /// native value changes each report `Begin` and `Commit`.
    #[track_caller]
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.editing = self.editing.on_edit(f);
        self
    }

    /// Move the focus to `target` when Enter or Escape ends the draft
    /// ([`draft`](Self::on_draft_commit)): the control the field was opened from.
    pub fn return_focus(mut self, target: NodeRef) -> Self {
        self.editing = self.editing.return_focus(target);
        self
    }

    /// Observe transitions into and out of native composition.
    pub fn on_composition_change<F, R>(mut self, f: F) -> Self
    where
        F: Fn(kivact_core::view::Composition) -> R + 'static,
        R: HandlerResult,
    {
        self.editing = self.editing.on_composition_change(f);
        self
    }
}

impl NodeBuilder for TextInput {
    /// The native input field itself, so `style`, `name`, `disabled`,
    /// `on_key` and the other modifiers apply to the field and win over the
    /// theme's look.
    fn node_mut(&mut self) -> &mut Node {
        &mut self.field
    }
}

impl IntoView for TextInput {
    fn into_view(mut self) -> View {
        if let Some(invalid) = self.field.accessible_invalid() {
            let _ = self.invalid.set(invalid);
        }
        self.field.into_view()
    }
}
