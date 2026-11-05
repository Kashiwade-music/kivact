//! Role: Render a themed number field with decrease and increase buttons.
use super::layer::with_recipe_style;
use super::text_input::text_input;
use super::variants::{Variant, Variants};
use super::{ButtonVariant, button};
use kivact_core::custom::PartState;
use kivact_core::i18n::{FractionDigits, Grouping, Locale, NumberStyle, message, use_locale};
use kivact_core::state::{Binding, Machine, Step, Value, binding, machine, memo};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::style::{Track, auto, style};
use kivact_core::view::EditPhase;
use kivact_core::view::{IntoView, View, node_ref};
use kivact_core::view::{Node, NodeBuilder, grid, icon};
use kivact_interaction::{ScalarRange, use_scalar};

/// What a number field shows: the value formatted, or what the user typed
/// while the field has the focus.
#[derive(Clone, PartialEq)]
enum NumberText {
    Value(String),
    /// `produced` is the stored value the last parsable text produced
    /// (`None` until one parses).
    Typed {
        text: String,
        produced: Option<f64>,
    },
}

impl NumberText {
    fn text(&self) -> &str {
        match self {
            NumberText::Value(text) | NumberText::Typed { text, .. } => text,
        }
    }
}

enum NumberEvent {
    ValueChanged,
    Typed(String),
    FocusOut,
}
struct NumberInputSnapshot {
    value: f64,
    range: ScalarRange,
    locale: Locale,
}
impl Machine for NumberInputSnapshot {
    type State = NumberText;
    type Event = NumberEvent;
    type Effect = ();
    fn transition(
        &self,
        state: &NumberText,
        event: NumberEvent,
    ) -> Step<NumberText, NumberEvent, ()> {
        match event {
            NumberEvent::ValueChanged
                if self.locale.parse_number(state.text()) == Some(self.value)
                    || matches!(state, NumberText::Typed { produced: Some(p), .. } if *p == self.value) =>
            {
                Step::stay()
            }
            NumberEvent::ValueChanged | NumberEvent::FocusOut => Step::to(NumberText::Value(
                format_value(self.value, self.range, &self.locale),
            )),
            NumberEvent::Typed(text) => {
                let parsed = self.locale.parse_number(&text);
                let produced = match parsed {
                    Some(_) => Some(self.value),
                    None => match state {
                        NumberText::Typed { produced, .. } => *produced,
                        NumberText::Value(_) => None,
                    },
                };
                Step::to(NumberText::Typed { text, produced })
            }
        }
    }
}

/// A number field with a decrease and an increase button, created by
/// [`number_input`].
pub struct NumberInput {
    root: Node,
    label: Value<String>,
    value: Binding<f64>,
    range: Value<ScalarRange>,
    on_edit: Option<kivact_core::state::Callback<EditPhase>>,
    variants: Variants,
}

/// A number field named `label` that edits `value` within `range`.
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
///     number_input("Quantity", value, range)
/// }
/// ```
#[track_caller]
pub fn number_input(
    label: impl IntoText,
    value: impl kivact_core::state::IntoBinding<f64>,
    range: impl IntoValue<ScalarRange>,
) -> NumberInput {
    let variants = Variants::default();
    let root = with_recipe_style(
        grid(()).base_style(style().grid_template_columns((auto(), Track::fr(1), auto()))),
        |r| r.number_input_recipes().root_recipe(),
        &variants,
        PartState::default,
    );
    NumberInput {
        root,
        label: label.into_text_value(),
        value: value.into_binding(),
        range: range.into_value(),
        on_edit: None,
        variants,
    }
}

impl NumberInput {
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

impl NodeBuilder for NumberInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
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

/// Keep the precision of the step and the bounds (the step grid starts at the
/// minimum and the maximum clamps), or the full value without a step.
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
impl IntoView for NumberInput {
    fn into_view(self) -> View {
        let NumberInput {
            mut root,
            label,
            value,
            range,
            on_edit,
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
        let mut number = use_scalar(value, range).disabled(disabled);
        if let Some(on_edit) = on_edit {
            number = number.on_edit(move |phase| on_edit.call(phase));
        }
        let shown = number.value();
        let locale = use_locale();
        let typed = machine(
            NumberText::Value(format_value(
                shown.get_untracked(),
                range.get_untracked(),
                &locale.get_untracked(),
            )),
            move || NumberInputSnapshot {
                value: shown.get(),
                range: range.get(),
                locale: locale.get(),
            },
            |_, ()| {},
        );
        let input_ref = node_ref();
        let focused = input_ref.focused();
        typed.feed(
            memo(move || (shown.get(), locale.get(), range.get())),
            |_| NumberEvent::ValueChanged,
        );
        let edit = binding(
            move || typed.get().text().to_owned(),
            move |text: String| {
                // Scalar owns the numeric write. Snapshot its accepted value
                // when recording text, including bindings that reject a write.
                if let Some(parsed) = locale.get_untracked().parse_number(&text) {
                    number.once(|| number.set(parsed));
                }
                typed.send(NumberEvent::Typed(text));
            },
        );
        let field = text_input(label, edit)
            .disabled(disabled)
            .invalid(move || {
                invalid.get()
                    || match typed.get() {
                        NumberText::Value(_) => false,
                        NumberText::Typed { text, .. } => {
                            let range = range.get();
                            locale.get().parse_number(&text).is_none_or(|value| {
                                !value.is_finite() || value < range.min() || value > range.max()
                            })
                        }
                    }
            })
            .with_node(|node| node.base_style(style().grid_row(1).grid_column(2)))
            .node_ref(input_ref)
            .with_node(|node| node.part(number.input()));
        // Leaving the field shows the written value again.
        kivact_core::state::effect_on(focused, move |&now| {
            if !now {
                typed.send(NumberEvent::FocusOut);
            }
        });
        let mut field = with_recipe_style(
            field,
            |r| r.number_input_recipes().input_recipe(),
            &variants,
            PartState::default,
        );
        for reference in labelled_by {
            field = field.with_node(|node| node.labelled_by(reference));
        }
        for reference in described_by {
            field = field.with_node(|node| node.described_by(reference));
        }
        let decrease = step_button(number, input_ref, disabled, false);
        let increase = step_button(number, input_ref, disabled, true);
        root.child((field, decrease, increase)).into_view()
    }
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
