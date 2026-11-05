//! Role: Ordered date-range editing, presets and shared range calendar.
use crate::{Calendar, DatePresentation, button, calendar, date_input, dialog};
use kivact_core::i18n::{CalendarDate, DateRange, DateStyle, Weekday, message};
use kivact_core::state::{Binding, IntoBinding, IntoText, IntoValue, Value, binding, signal};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, column, node, row};
use std::rc::Rc;
/// A named range evaluated when the user activates it.
#[derive(Clone)]
pub struct DateRangePreset {
    label: Value<String>,
    range: Rc<dyn Fn() -> DateRange>,
}
impl PartialEq for DateRangePreset {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.range, &other.range)
    }
}
impl DateRangePreset {
    /// Create a preset whose dates may depend on the current day or application state.
    pub fn new(label: impl IntoText, range: impl Fn() -> DateRange + 'static) -> Self {
        Self {
            label: label.into_text_value(),
            range: Rc::new(range),
        }
    }
}
type Constraint = Rc<dyn Fn(CalendarDate) -> bool>;
/// A range field whose endpoints always form an ordered [`DateRange`].
pub struct DateRangeInput {
    root: Node,
    variants: crate::variants::Variants,
    label: Value<String>,
    start_label: Value<String>,
    end_label: Value<String>,
    value: Binding<Option<DateRange>>,
    format: Value<DateStyle>,
    custom_format: Option<Rc<dyn Fn(CalendarDate) -> String>>,
    placeholder: Option<Value<String>>,
    valid: Option<Binding<bool>>,
    min: Value<Option<CalendarDate>>,
    max: Value<Option<CalendarDate>>,
    week_start: Option<Value<Weekday>>,
    presentation: DatePresentation,
    configure: Option<Rc<dyn Fn(Calendar) -> Calendar>>,
    presets: Option<Value<Vec<DateRangePreset>>>,
    constraints: Vec<Constraint>,
    min_span: Value<u32>,
    max_span: Value<Option<u32>>,
}
/// Edit two dates; selecting an earlier end reorders the range in one transition.
pub fn date_range_input(
    label: impl IntoText,
    range: impl IntoBinding<Option<DateRange>>,
) -> DateRangeInput {
    DateRangeInput {
        root: node(),
        variants: crate::variants::Variants::default(),
        label: label.into_text_value(),
        start_label: message("date.start", "Start date"),
        end_label: message("date.end", "End date"),
        value: range.into_binding(),
        format: DateStyle::Iso.into_value(),
        custom_format: None,
        placeholder: None,
        valid: None,
        min: None.into_value(),
        max: None.into_value(),
        week_start: None,
        presentation: DatePresentation::Adaptive,
        configure: None,
        presets: None,
        constraints: Vec::new(),
        min_span: 1.into_value(),
        max_span: None.into_value(),
    }
}
impl DateRangeInput {
    /// Add a visual variant shared by the field and its picker surface.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
    /// Set the accessible label of the start-date editor.
    pub fn start_label(mut self, label: impl IntoText) -> Self {
        self.start_label = label.into_text_value();
        self
    }
    /// Set the accessible label of the end-date editor.
    pub fn end_label(mut self, label: impl IntoText) -> Self {
        self.end_label = label.into_text_value();
        self
    }

    /// Format both endpoints.
    pub fn format(mut self, format: impl IntoValue<DateStyle>) -> Self {
        self.format = format.into_value();
        self.custom_format = None;
        self
    }
    /// Custom application date presentation for both endpoints.
    pub fn format_with(mut self, format: impl Fn(CalendarDate) -> String + 'static) -> Self {
        self.custom_format = Some(Rc::new(format));
        self
    }
    /// Empty endpoint text.
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = Some(text.into_text_value());
        self
    }
    /// Publish whether both endpoint drafts are valid.
    pub fn valid(mut self, valid: impl IntoBinding<bool>) -> Self {
        self.valid = Some(valid.into_binding());
        self
    }
    /// Earliest accepted endpoint.
    pub fn min(mut self, min: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.min = min.into_value();
        self
    }
    /// Latest accepted endpoint.
    pub fn max(mut self, max: impl IntoValue<Option<CalendarDate>>) -> Self {
        self.max = max.into_value();
        self
    }
    /// First weekday column.
    pub fn week_start(mut self, day: impl IntoValue<Weekday>) -> Self {
        self.week_start = Some(day.into_value());
        self
    }
    /// Choose desktop or touch presentation.
    pub fn presentation(mut self, presentation: DatePresentation) -> Self {
        self.presentation = presentation;
        self
    }
    /// Configure the range calendar.
    pub fn calendar(mut self, configure: impl Fn(Calendar) -> Calendar + 'static) -> Self {
        self.configure = Some(Rc::new(configure));
        self
    }
    /// Display quick range choices.
    pub fn presets(mut self, presets: impl IntoValue<Vec<DateRangePreset>>) -> Self {
        self.presets = Some(presets.into_value());
        self
    }
    /// Require both endpoints to satisfy this predicate.
    pub fn date_constraint(mut self, constraint: impl Fn(CalendarDate) -> bool + 'static) -> Self {
        self.constraints.push(Rc::new(constraint));
        self
    }
    /// Minimum inclusive span in days.
    pub fn min_range_span(mut self, span: impl IntoValue<u32>) -> Self {
        self.min_span = span.into_value();
        self
    }
    /// Maximum inclusive span in days.
    pub fn max_range_span(mut self, span: impl IntoValue<u32>) -> Self {
        let span = span.into_value();
        self.max_span = (move || Some(span.get())).into_value();
        self
    }
}
impl NodeBuilder for DateRangeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for DateRangeInput {
    fn into_view(self) -> View {
        self.render(crate::date_trigger::FieldPresentation::Text)
    }
}
impl DateRangeInput {
    /// Open the shared range picker from a caller-owned trigger, without endpoint editors.
    pub fn attach<T: NodeBuilder + IntoView + 'static>(self, trigger: T) -> View {
        self.render(crate::date_trigger::FieldPresentation::trigger(trigger))
    }
    fn render(self, field: crate::date_trigger::FieldPresentation) -> View {
        let Self {
            root,
            variants,
            label,
            start_label,
            end_label,
            value,
            format,
            custom_format,
            placeholder,
            valid,
            min,
            max,
            week_start,
            presentation,
            configure,
            presets,
            constraints,
            min_span,
            max_span,
        } = self;
        let configured_rules = configure
            .as_ref()
            .map(|configure| configure(calendar(value)).range_validator());
        let constraints = Rc::new(constraints);
        let accept: Rc<dyn Fn(DateRange) -> bool> = Rc::new(move |range| {
            let (start, end) = (range.start(), range.end());
            let span = start.to_days().abs_diff(end.to_days()) + 1;
            configured_rules.as_ref().is_none_or(|accept| accept(range))
                && min.get().is_none_or(|min| start >= min)
                && max.get().is_none_or(|max| end <= max)
                && span >= u64::from(min_span.get().max(1))
                && max_span.get().is_none_or(|max| span <= u64::from(max))
                && constraints
                    .iter()
                    .all(|accept| accept(start) && accept(end))
        });
        let accepted = accept.clone();
        let ranged = binding(value, move |next: Option<DateRange>| {
            if next.is_none_or(|range| accepted(range)) {
                value.set(next);
            }
        });
        let open = signal(false);
        let trigger_focus = match &field {
            crate::date_trigger::FieldPresentation::Trigger(attach) => Some(attach.focus),
            crate::date_trigger::FieldPresentation::Text => None,
        };
        let content = move || {
            let local = signal(value.get_untracked());
            let accepted = accept.clone();
            let mut picker = calendar(local)
                .min(min)
                .max(max)
                .range_span(min_span, max_span)
                .number_of_months(2usize)
                .on_commit(move || {
                    if let Some(next) = local.get_untracked().filter(|next| accepted(*next)) {
                        value.set(Some(next));
                        open.set(false);
                        if let Some(focus) = trigger_focus {
                            focus.restore_focus();
                        }
                    }
                });
            if let Some(day) = week_start {
                picker = picker.week_start(day);
            }
            if let Some(configure) = &configure {
                picker = configure(picker);
            }
            let accepted = accept.clone();
            let choices = kivact_core::view::dynamic(
                move || presets.map(|presets| presets.get()).unwrap_or_default(),
                move |presets| {
                    column(
                        presets
                            .into_iter()
                            .map(|preset| {
                                let accepted = accepted.clone();
                                button(preset.label).on_press(move || {
                                    let range = (preset.range)();
                                    if accepted(range) {
                                        value.set(Some(range));
                                        open.set(false);
                                        if let Some(focus) = trigger_focus {
                                            focus.restore_focus();
                                        }
                                    }
                                })
                            })
                            .collect::<Vec<_>>(),
                    )
                },
            );
            column((choices, picker))
        };

        if let crate::date_trigger::FieldPresentation::Trigger(attach) = field {
            return root
                .child(
                    attach.view(
                        dialog(label, open)
                            .with_variants(&variants)
                            .mode(presentation.mode())
                            .hide_title(true)
                            .content(content),
                    ),
                )
                .into_view();
        }
        let start = binding(
            move || value.get().map(DateRange::start),
            move |next| {
                ranged.set(next.map(|start| {
                    DateRange::new(start, value.get_untracked().map_or(start, DateRange::end))
                }))
            },
        );
        let end = binding(
            move || value.get().map(DateRange::end),
            move |next| {
                ranged.set(next.map(|end| {
                    DateRange::new(value.get_untracked().map_or(end, DateRange::start), end)
                }))
            },
        );
        let validation = kivact_core::state::machine(
            RangeValidation::Valid,
            || RangeValidator,
            move |_, next| {
                if let Some(valid) = valid {
                    valid.set(next);
                }
            },
        );
        let start_valid = binding(
            move || {
                matches!(
                    validation.get(),
                    RangeValidation::Valid | RangeValidation::EndInvalid
                )
            },
            move |valid| validation.send(ValidationEvent::Start(valid)),
        );
        let end_valid = binding(
            move || {
                matches!(
                    validation.get(),
                    RangeValidation::Valid | RangeValidation::StartInvalid
                )
            },
            move |valid| validation.send(ValidationEvent::End(valid)),
        );
        let mut first = date_input(start_label, start)
            .format(format)
            .min(min)
            .max(max)
            .presentation(presentation)
            .with_variants(&variants);
        let mut last = date_input(end_label, end)
            .format(format)
            .min(min)
            .max(max)
            .presentation(presentation)
            .with_variants(&variants);
        first = first.valid(start_valid);
        last = last.valid(end_valid);
        if let Some(placeholder) = placeholder {
            first = first.placeholder(placeholder);
            last = last.placeholder(placeholder);
        }
        if let Some(format) = custom_format {
            let first_format = format.clone();
            first = first.format_with(move |date| first_format(date));
            last = last.format_with(move |date| format(date));
        }
        if let Some(day) = week_start {
            first = first.week_start(day);
            last = last.week_start(day);
        }
        let root = crate::layer::with_base_recipe_style(
            root,
            |r| r.date_range_input_recipes().root_recipe(),
            &variants,
            kivact_core::custom::PartState::default,
        );
        let trigger = button(message("date.range_open", "Choose date range"));
        root.child(row((
            first,
            last,
            dialog(label, open)
                .with_variants(&variants)
                .mode(presentation.mode())
                .hide_title(true)
                .content(content)
                .attach(trigger),
        )))
        .into_view()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RangeValidation {
    Valid,
    StartInvalid,
    EndInvalid,
    BothInvalid,
}
enum ValidationEvent {
    Start(bool),
    End(bool),
}
struct RangeValidator;
impl kivact_core::state::Machine for RangeValidator {
    type State = RangeValidation;
    type Event = ValidationEvent;
    type Effect = bool;
    fn transition(
        &self,
        state: &RangeValidation,
        event: ValidationEvent,
    ) -> kivact_core::state::Step<RangeValidation, ValidationEvent, bool> {
        use RangeValidation::{BothInvalid, EndInvalid, StartInvalid, Valid};
        let next = match (state, event) {
            (Valid | StartInvalid, ValidationEvent::Start(true))
            | (Valid | EndInvalid, ValidationEvent::End(true)) => Valid,
            (EndInvalid | BothInvalid, ValidationEvent::Start(true)) => EndInvalid,
            (StartInvalid | BothInvalid, ValidationEvent::End(true)) => StartInvalid,
            (Valid | StartInvalid, ValidationEvent::Start(false)) => StartInvalid,
            (Valid | EndInvalid, ValidationEvent::End(false)) => EndInvalid,
            (_, ValidationEvent::Start(false) | ValidationEvent::End(false)) => BothInvalid,
        };
        kivact_core::state::Step::to(next).with(next == Valid)
    }
}
