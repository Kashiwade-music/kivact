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
    label: Value<String>,
    value: Binding<Option<DateRange>>,
    format: Value<DateStyle>,
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
        label: label.into_text_value(),
        value: range.into_binding(),
        format: DateStyle::Iso.into_value(),
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
    /// Format both endpoints.
    pub fn format(mut self, format: impl IntoValue<DateStyle>) -> Self {
        self.format = format.into_value();
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
        let Self {
            root,
            label,
            value,
            format,
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
        let constraints = Rc::new(constraints);
        let accept: Rc<dyn Fn(DateRange) -> bool> = Rc::new(move |range| {
            let (start, end) = (range.start(), range.end());
            let span = start.to_days().abs_diff(end.to_days()) + 1;
            min.get().is_none_or(|min| start >= min)
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
        let mut first = date_input(message("date.start", "Start date"), start)
            .format(format)
            .min(min)
            .max(max)
            .presentation(presentation);
        let mut last = date_input(message("date.end", "End date"), end)
            .format(format)
            .min(min)
            .max(max)
            .presentation(presentation);
        if let Some(day) = week_start {
            first = first.week_start(day);
            last = last.week_start(day);
        }
        let open = signal(false);
        let trigger = row((
            first,
            last,
            button(message("date.range_open", "Choose date range"))
                .on_press(move || open.set(true)),
        ));
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
                                    }
                                })
                            })
                            .collect::<Vec<_>>(),
                    )
                },
            );
            column((choices, picker))
        };
        let root = crate::layer::with_base_recipe_style(
            root,
            |r| r.date_range_input_recipes().root_recipe(),
            &crate::variants::Variants::default(),
            kivact_core::custom::PartState::default,
        );
        root.child(
            dialog(label, open)
                .mode(presentation.mode())
                .hide_title(true)
                .content(content)
                .attach(trigger),
        )
        .into_view()
    }
}
