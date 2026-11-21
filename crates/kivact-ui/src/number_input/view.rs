//! Role: The number input builder, its draft text and its rendering.

use crate::number_input::look::NumberInputTheme;

use crate::helpers::{is_named, report_unnamed};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::text_input::{TextInput, text_input};
use crate::{ButtonVariant, button};
use kivact_core::custom::PartState;
use kivact_core::i18n::{FractionDigits, Grouping, Locale, NumberStyle, message, use_locale};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::state::{Signal, Value, binding};
use kivact_core::style::{Track, auto, style};
use kivact_core::view::EditPhase;
use kivact_core::view::Variants;
use kivact_core::view::{Element, Node, grid, icon};
use kivact_core::view::{IntoView, NodeRef, View, node_ref};
use kivact_interaction::{DraftCommit, DraftOptions, ScalarRange, use_draft, use_scalar};
use std::panic::Location;

/// A number field with a decrease and an increase button, created by
/// [`number_input`].
pub struct NumberInput {
    root: Node,
    value: Signal<f64>,
    range: Value<ScalarRange>,
    on_edit: Option<kivact_core::state::Callback<EditPhase>>,
    at: &'static Location<'static>,
}

/// A number field that edits `value` within `range`. Name it with
/// [`label`](NumberInput::label), or with [`field`](crate::field).
///
/// The text shown is the value; each text that parses to a number is written
/// at once, clamped to the range and rounded to its step. The text the user
/// typed stays while the field has the focus, and the field shows the written
/// value when it loses the focus. Arrow Up and Arrow Down
/// change the value by one step, Page Up and Page Down by ten, and Home and End
/// go to the bounds ([`use_scalar`]'s keys of a spin button). The buttons,
/// named by the messages `stepper.decrease` ("Decrease") and
/// `stepper.increase` ("Increase"), change it by one step: the range's step,
/// or 1 when it has none.
///
/// The editor precedes the step buttons in focus/semantic order, so activating
/// a surrounding field label focuses the value without stepping it. Structural
/// grid placement keeps the visible order decrease, editor, increase. Step buttons
/// keep focus on the editor and do not add tab stops. Invalid typed text is
/// exposed as invalid until it is corrected or the field loses focus. Disabled
/// and invalid state published on the root also reaches the editor.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::interaction::ScalarRange;
/// use kivact::ui::number_input;
///
/// fn quantity() -> impl IntoView {
///     let value = signal(2.0);
///     let range = ScalarRange::new(0.0, 10.0).unwrap().with_step(1.0).unwrap();
///     number_input(value, range).label("Quantity")
/// }
/// ```
#[track_caller]
pub fn number_input(
    value: kivact_core::state::Signal<f64>,
    range: impl IntoValue<ScalarRange>,
) -> NumberInput {
    let root = grid(()).base_style(style().grid_template_columns((auto(), Track::fr(1), auto())));
    NumberInput {
        root,
        value,
        range: range.into_value(),
        on_edit: None,
        at: Location::caller(),
    }
}

impl NumberInput {
    /// The editor's accessible name (the root's name, as `.name` sets it).
    /// Inside a [`field`](crate::field) the field's label names it.
    #[track_caller]
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.root = self.root.name(label);
        self
    }

    /// Report each key press, button press or written text as one edit
    /// (`Begin`, the write, `Commit`), as every standard component that
    /// writes a value does.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(kivact_core::state::callback(f));
        self
    }
}

impl Element for NumberInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// Keep the precision of the step and the bounds (the step grid starts at the
/// minimum and the maximum clamps), or the full value without a step.
impl IntoView for NumberInput {
    fn into_view(self) -> View {
        let NumberInput {
            mut root,
            value,
            range,
            on_edit,
            at,
        } = self;
        let variants = root.authored().variants();
        let disabled = root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let label = editor_name(&mut root, at);
        let invalid = root
            .accessible_invalid()
            .unwrap_or_else(|| false.into_value());
        let labelled_by = root.accessible_labelled_by();
        let described_by = root.accessible_described_by();
        let mut number = use_scalar(value, range).disabled(disabled);
        if let Some(on_edit) = on_edit {
            number = number.on_edit(move |phase| on_edit.call(phase));
        }
        let shown = number.value();
        let locale = use_locale();
        // Scalar owns the numeric write; the draft writes its accepted value through it.
        let writer = binding(shown, move |next: f64| {
            number.once(|| number.set(next));
        });
        let draft = use_draft(
            writer,
            DraftOptions::new(
                move |value: &f64| format_value(*value, range.get(), &locale.get()),
                move |text: &String| locale.get().parse_number(text),
            )
            .commit(DraftCommit::Live),
        );
        let input_ref = node_ref();
        // Leaving the field shows the written value again.
        draft.settle_on_blur(input_ref.focused());
        let field = text_input(draft.editor())
            .label(label)
            .disabled(disabled)
            .invalid(move || {
                invalid.get()
                    || (draft.is_editing().get()
                        && text_invalid(&draft.draft().get(), range.get(), &locale.get()))
            })
            .with_node(|node| node.base_style(style().grid_row(1).grid_column(2)))
            .node_ref(input_ref)
            .with_node(|node| node.part(number.input()));
        let field = editor_field(field, &variants, labelled_by, described_by);
        let decrease = step_button(number, input_ref, disabled, false);
        let increase = step_button(number, input_ref, disabled, true);
        with_base_recipe_style(
            root,
            |r: &NumberInputTheme| r.root_recipe(),
            &variants,
            PartState::default,
        )
        .child((field, decrease, increase))
        .into_view()
    }
}

/// The editor's name: the root's name. Without one (and without a
/// `field(..)`) the editor is named "" and the missing name is reported.
fn editor_name(root: &mut Node, at: &'static Location<'static>) -> Value<String> {
    report_unnamed(is_named(root), "number_input", at);
    root.accessible_name()
        .unwrap_or_else(|| String::new().into_value())
}

/// The editor with its recipe and the caller's accessibility references.
fn editor_field(
    mut field: TextInput,
    variants: &Variants,
    labelled_by: Vec<NodeRef>,
    described_by: Vec<NodeRef>,
) -> TextInput {
    field = with_recipe_style(
        field,
        |r: &NumberInputTheme| r.input_recipe(),
        variants,
        PartState::default,
    );
    for reference in labelled_by {
        field = field.with_node(|node| node.labelled_by(reference));
    }
    for reference in described_by {
        field = field.with_node(|node| node.described_by(reference));
    }
    field
}

/// Render a spinbutton step action while keeping focus in its editor.
fn step_button(
    number: kivact_interaction::ScalarApi,
    input_ref: kivact_core::view::NodeRef,
    disabled: Value<bool>,
    increase: bool,
) -> impl IntoView {
    let (label, glyph, column, steps) = if increase {
        (message("stepper.increase", "Increase"), "plus", 3, 1.0)
    } else {
        (message("stepper.decrease", "Decrease"), "minus", 1, -1.0)
    };
    button(label)
        .with_node(|node| {
            node.base_style(style().grid_row(1).grid_column(column))
                .focusable(false)
                .focus_on_press(input_ref)
        })
        .disabled(disabled)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon(glyph))
        .on_press(move || {
            number.once(|| number.step_by(steps));
        })
}

/// Validate a pending textual edit against the caller's current numeric bounds.
fn text_invalid(text: &str, range: ScalarRange, locale: &Locale) -> bool {
    locale
        .parse_number(text)
        .is_none_or(|value| !value.is_finite() || value < range.min() || value > range.max())
}

fn format_value(value: f64, range: ScalarRange, locale: &Locale) -> String {
    let places = range.step().map_or_else(
        || places(value),
        |step| {
            places(step)
                .max(places(range.min()))
                .max(places(range.max()))
        },
    );
    locale.format_number(
        value,
        NumberStyle {
            fraction_digits: FractionDigits::Range {
                min: 0,
                max: places.min(255) as u8,
            },
            grouping: Grouping::None,
        },
    )
}

/// Decimal places required by the step, including a decimal exponent.
fn places(step: f64) -> usize {
    let text = step.to_string();
    let (mantissa, exponent) =
        text.split_once('e')
            .map_or((text.as_str(), 0), |(mantissa, exponent)| {
                (
                    mantissa,
                    exponent
                        .parse::<i32>()
                        .expect("invariant: f64 exponent is an integer"),
                )
            });
    let decimals = mantissa
        .split_once('.')
        .map_or(0, |(_, decimals)| decimals.len());
    (decimals as i32 - exponent).max(0) as usize
}
