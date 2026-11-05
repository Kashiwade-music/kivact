//! Role: Time field and its wheel dialog.
use super::machine::{Event, Snapshot, TimeDraft, TimeInputState};
use crate::layer::with_recipe_style;
use crate::{DatePresentation, button, dialog, text_input};
use kivact_core::custom::PartState;
use kivact_core::i18n::{TimeOfDay, TimeStyle, message, use_locale};
use kivact_core::state::{Binding, IntoBinding, IntoText, IntoValue, Value, binding, machine};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, node, row};
/// A wall-clock time field with hours and minutes, optionally seconds.
pub struct TimeInput {
    root: Node,
    variants: crate::variants::Variants,
    label: Value<String>,
    value: Binding<Option<TimeOfDay>>,
    seconds: Value<bool>,
    min: Value<Option<TimeOfDay>>,
    max: Value<Option<TimeOfDay>>,
    presentation: DatePresentation,
}
/// Edit optional wall-clock time, independently of date and time zone.
pub fn time_input(label: impl IntoText, value: impl IntoBinding<Option<TimeOfDay>>) -> TimeInput {
    TimeInput {
        root: node(),
        variants: crate::variants::Variants::default(),
        label: label.into_text_value(),
        value: value.into_binding(),
        seconds: false.into_value(),
        min: None.into_value(),
        max: None.into_value(),
        presentation: DatePresentation::Adaptive,
    }
}
impl TimeInput {
    /// Add a visual variant shared by the field and its picker surface.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
    pub(crate) fn with_variants(self, variants: &crate::variants::Variants) -> Self {
        self.variants.merge(variants);
        self
    }
    /// Format the wall-clock time, read reactively.
    pub fn format(mut self, format: impl IntoValue<TimeStyle>) -> Self {
        let format = format.into_value();
        self.seconds = (move || format.get().seconds).into_value();
        self
    }
    /// Earliest accepted wall-clock time.
    pub fn min(mut self, min: impl IntoValue<Option<TimeOfDay>>) -> Self {
        self.min = min.into_value();
        self
    }
    /// Latest accepted wall-clock time.
    pub fn max(mut self, max: impl IntoValue<Option<TimeOfDay>>) -> Self {
        self.max = max.into_value();
        self
    }
    /// Choose popup or touch sheet.
    pub fn presentation(mut self, presentation: DatePresentation) -> Self {
        self.presentation = presentation;
        self
    }
}
impl NodeBuilder for TimeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for TimeInput {
    fn into_view(self) -> View {
        let Self {
            root,
            variants,
            label,
            value,
            seconds,
            min,
            max,
            presentation,
        } = self;
        let locale = use_locale();
        let initial = value
            .get_untracked()
            .map_or(TimeDraft::Empty, |value| TimeDraft::Valid {
                text: String::new(),
                value,
            });
        let state = machine(
            TimeInputState::Closed(initial),
            move || Snapshot {
                seconds: seconds.get(),
                min: min.get(),
                max: max.get(),
            },
            move |_, next| value.set(next),
        );
        kivact_core::state::effect(move || state.send(Event::External(value.get())));
        let draft = binding(
            move || match state.get().draft() {
                TimeDraft::Invalid(text) => text.clone(),
                TimeDraft::Valid { text, value: _ } if !text.is_empty() => text.clone(),
                _ => value
                    .get()
                    .map(|value| {
                        locale.get().format_time(
                            value,
                            TimeStyle {
                                seconds: seconds.get(),
                            },
                        )
                    })
                    .unwrap_or_default(),
            },
            move |text| state.send(Event::Text(text)),
        );
        let field = text_input(label, draft)
            .placeholder("HH:MM")
            .invalid(move || matches!(state.get().draft(), TimeDraft::Invalid(_)))
            .on_draft_commit(move |text| state.send(Event::Commit(text)));
        let toggle =
            button(message("time.open", "Choose time")).on_press(move || state.send(Event::Open));
        let open = binding(
            move || state.get().is_open(),
            move |open| state.send(if open { Event::Open } else { Event::Close }),
        );
        let picked = binding(value, move |next| state.send(Event::Pick(next)));
        root.child(with_recipe_style(
            row((
                field,
                dialog(label, open)
                    .with_variants(&variants)
                    .mode(presentation.mode())
                    .hide_title(true)
                    .content(move || crate::date_wheel::time_panel(picked, seconds))
                    .attach(toggle),
            )),
            |r| r.time_input_recipes().root_recipe(),
            &variants,
            PartState::default,
        ))
        .into_view()
    }
}
