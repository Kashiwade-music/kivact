//! Role: Ordered date-range editing, presets and shared range calendar.
use crate::date_range_draft::{
    Effect as DraftEffect, Event as DraftEvent, FieldValidity, Policy as DraftPolicy, RangeDraft,
};
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
    picker_label: Option<Value<String>>,
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
        picker_label: None,
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
    /// Name the separate picker button independently of the text editor.
    pub fn picker_label(mut self, label: impl IntoText) -> Self {
        self.picker_label = Some(label.into_text_value());
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

    /// Add a visual variant shared by the field and its picker surface.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
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
        let accept = self.range_validator();
        let open = signal(false);
        let trigger_focus = match &field {
            crate::date_trigger::FieldPresentation::Trigger(attach) => Some(attach.focus),
            crate::date_trigger::FieldPresentation::Text => None,
        };
        let content = self.picker_content(open, accept.clone(), trigger_focus);
        if let crate::date_trigger::FieldPresentation::Trigger(attach) = field {
            return self
                .root
                .child(
                    attach.view(
                        dialog(self.label, open)
                            .with_variants(&self.variants)
                            .mode(self.presentation.mode())
                            .hide_title(true)
                            .content(move || content.call(())),
                    ),
                )
                .into_view();
        }
        let editors = range_editors(self.value, self.valid, accept);
        let (order_ref, order_error) = order_diagnostic(editors.draft);
        let (first, last) = self.endpoint_fields(&editors, order_ref);
        let root = crate::layer::with_base_recipe_style(
            self.root,
            |r| r.date_range_input_recipes().root_recipe(),
            &self.variants,
            kivact_core::custom::PartState::default,
        );
        let trigger = button(
            self.picker_label
                .unwrap_or_else(|| message("date.range_open", "Choose date range")),
        );
        root.child(row((
            first,
            last,
            dialog(self.label, open)
                .with_variants(&self.variants)
                .mode(self.presentation.mode())
                .hide_title(true)
                .content(move || content.call(()))
                .attach(trigger),
        )))
        .child(order_error)
        .into_view()
    }
}

impl DateRangeInput {
    /// Compose configured calendar, endpoint, span and caller constraints into one validator.
    fn range_validator(&self) -> Rc<dyn Fn(DateRange) -> bool> {
        let (min, max, min_span, max_span) = (self.min, self.max, self.min_span, self.max_span);
        let configured_rules = self
            .configure
            .as_ref()
            .map(|configure| configure(calendar(self.value)).range_validator());
        let constraints = Rc::new(self.constraints.clone());
        Rc::new(move |range| {
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
        })
    }
    /// Render preset choices and the canonical two-month range picker.
    fn picker_content(
        &self,
        open: kivact_core::state::Signal<bool>,
        accept: Rc<dyn Fn(DateRange) -> bool>,
        trigger_focus: Option<kivact_core::view::NodeRef>,
    ) -> kivact_core::view::ViewFn {
        let (min, max, min_span, max_span) = (self.min, self.max, self.min_span, self.max_span);
        let value = self.value;
        let week_start = self.week_start;
        let configure = self.configure.clone();
        let presets = self.presets;
        let touch = self.presentation.sheet();
        kivact_core::view::ViewFn::from(move |()| {
            let selected = committing_range(value, accept.clone(), open, trigger_focus);
            let mut picker = calendar(selected)
                .min(min)
                .max(max)
                .range_span(min_span, max_span)
                .number_of_months(2usize);
            if touch {
                picker = picker.touch();
            }
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
                            .enumerate()
                            .map(|(index, preset)| {
                                let accepted = accepted.clone();
                                let range = preset.range.clone();
                                let selected_range = range.clone();
                                let disabled_range = range.clone();
                                let enabled = accepted.clone();
                                let reference = kivact_core::view::node_ref();
                                if index == 0 {
                                    kivact_core::state::on_mount(move || reference.focus());
                                }
                                button(preset.label)
                                    .node_ref(reference)
                                    .with_node(|node| {
                                        node.selected(move || value.get() == Some(selected_range()))
                                    })
                                    .disabled(move || !enabled(disabled_range()))
                                    .on_press(move || {
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
            row((
                kivact_core::view::node()
                    .child(choices)
                    .style(kivact_core::style::style().width(180)),
                picker,
            ))
        })
    }
    /// Render independently validated endpoint editors with the shared order diagnostic.
    fn endpoint_fields(
        &self,
        editors: &RangeEditors,
        order_ref: kivact_core::view::NodeRef,
    ) -> (crate::DateInput, crate::DateInput) {
        let RangeEditors {
            draft,
            start,
            end,
            start_valid,
            end_valid,
        } = *editors;
        let (start_label, end_label, format, min, max, presentation, placeholder, week_start) = (
            self.start_label,
            self.end_label,
            self.format,
            self.min,
            self.max,
            self.presentation,
            self.placeholder,
            self.week_start,
        );
        let variants = &self.variants;
        let mut first = date_input(start_label, start)
            .format(format)
            .min(min)
            .max(max)
            .presentation(presentation)
            .with_variants(variants);
        let mut last = date_input(end_label, end)
            .format(format)
            .min(min)
            .max(max)
            .presentation(presentation)
            .with_variants(variants);
        first = first
            .valid(start_valid)
            .invalid(move || draft.get().order_invalid())
            .with_node(|node| node.described_by(order_ref));
        last = last
            .valid(end_valid)
            .invalid(move || draft.get().order_invalid())
            .with_node(|node| node.described_by(order_ref));
        if let Some(placeholder) = placeholder {
            first = first.placeholder(placeholder);
            last = last.placeholder(placeholder);
        }
        if let Some(format) = self.custom_format.clone() {
            let first_format = format.clone();
            first = first.format_with(move |date| first_format(date));
            last = last.format_with(move |date| format(date));
        }
        if let Some(day) = week_start {
            first = first.week_start(day);
            last = last.week_start(day);
        }
        (first, last)
    }
}
// xtask-allow(state-fields): Only projections of one authoritative RangeDraft machine into endpoint and validity bindings; no separate mutable lifecycle state.
/// The canonical range draft and the bindings which project its endpoint fields.
#[derive(Clone, Copy)]
struct RangeEditors {
    draft: kivact_core::state::MachineHandle<DraftPolicy>,
    start: Binding<Option<CalendarDate>>,
    end: Binding<Option<CalendarDate>>,
    start_valid: Binding<bool>,
    end_valid: Binding<bool>,
}
/// Project endpoint edits and validity into the single canonical range draft machine.
fn range_editors(
    value: Binding<Option<DateRange>>,
    valid: Option<Binding<bool>>,
    accept: Rc<dyn Fn(DateRange) -> bool>,
) -> RangeEditors {
    let draft = kivact_core::state::machine(
        RangeDraft::Following(value.get_untracked()),
        move || DraftPolicy {
            accept: accept.clone(),
        },
        move |_, effect| match effect {
            DraftEffect::Publish(next) => value.set(next),
            DraftEffect::Validity(next) => {
                if let Some(valid) = valid {
                    valid.set(next)
                }
            }
        },
    );
    kivact_core::state::effect(move || draft.send(DraftEvent::External(value.get())));
    let start_value = kivact_core::state::memo(move || draft.get().endpoints().0);
    let end_value = kivact_core::state::memo(move || draft.get().endpoints().1);
    let start = binding(start_value, move |value| {
        draft.send(DraftEvent::Start(value))
    });
    let end = binding(end_value, move |value| draft.send(DraftEvent::End(value)));
    let start_valid = binding(
        move || {
            matches!(
                draft.get().fields(),
                FieldValidity::Valid | FieldValidity::EndInvalid
            )
        },
        move |valid| draft.send(DraftEvent::StartValid(valid)),
    );
    let end_valid = binding(
        move || {
            matches!(
                draft.get().fields(),
                FieldValidity::Valid | FieldValidity::StartInvalid
            )
        },
        move |valid| draft.send(DraftEvent::EndValid(valid)),
    );
    RangeEditors {
        draft,
        start,
        end,
        start_valid,
        end_valid,
    }
}
/// Expose invalid endpoint order from the actual range draft's validation state.
fn order_diagnostic(
    draft: kivact_core::state::MachineHandle<DraftPolicy>,
) -> (kivact_core::view::NodeRef, Node) {
    let order_ref = kivact_core::view::node_ref();
    let order_message = message("date.error.order", "The end date is before the start date");
    let order_error = kivact_core::view::text(order_message)
        .node_ref(order_ref)
        .role(kivact_core::view::Role::Status)
        .live_region(kivact_core::view::LiveRegion::Polite)
        .hidden(move || !draft.get().order_invalid())
        .style(
            kivact_core::style::style()
                .font_size(12)
                .color(kivact_core::style::ColorToken::Danger),
        );
    (order_ref, order_error)
}

fn committing_range(
    value: Binding<Option<DateRange>>,
    accept: Rc<dyn Fn(DateRange) -> bool>,
    open: kivact_core::state::Signal<bool>,
    focus: Option<kivact_core::view::NodeRef>,
) -> Binding<Option<DateRange>> {
    binding(value, move |next: Option<DateRange>| {
        if next.is_none_or(|next| accept(next)) {
            value.set(next);
            if value.get_untracked() == next {
                open.set(false);
                if let Some(focus) = focus {
                    focus.restore_focus();
                }
            }
        }
    })
}
