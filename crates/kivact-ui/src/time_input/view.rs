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
    picker_label: Option<Value<String>>,
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
        picker_label: None,
        value: value.into_binding(),
        seconds: false.into_value(),
        min: None.into_value(),
        max: None.into_value(),
        presentation: DatePresentation::Adaptive,
    }
}
impl TimeInput {
    /// Name the separate picker button independently of the text editor.
    pub fn picker_label(mut self, label: impl IntoText) -> Self {
        self.picker_label = Some(label.into_text_value());
        self
    }

    /// Add a visual variant shared by the field and its picker surface.
    pub fn variant(mut self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
    pub(crate) fn with_variants(mut self, variants: &crate::variants::Variants) -> Self {
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
            picker_label,
            value,
            seconds,
            min,
            max,
            presentation,
        } = self;
        let locale = use_locale();
        let state = time_controller(value, seconds, min, max);
        let draft = time_text(state, value, seconds, locale);
        let input_ref = kivact_core::view::node_ref();
        let button_ref = kivact_core::view::node_ref();
        let return_focus = kivact_core::state::memo(move || match state.get() {
            TimeInputState::Open { invoker, .. } => invoker,
            _ => button_ref,
        });
        let open = binding(
            move || state.get().is_open(),
            move |open| {
                state.send(if open {
                    Event::Open(button_ref)
                } else {
                    Event::Close
                })
            },
        );
        let field = text_input(label, draft)
            .node_ref(input_ref)
            .with_node(|n| n.expanded(open))
            .placeholder("HH:MM")
            .invalid(move || matches!(state.get().draft(), TimeDraft::Invalid(_)))
            .forward_keys(vec![kivact_core::view::Key::ArrowDown])
            .on_key(move |event: kivact_core::view::KeyEvent| {
                if event.key() == kivact_core::view::Key::ArrowDown {
                    state.send(Event::Open(input_ref));
                    kivact_core::view::Handled::Yes
                } else {
                    kivact_core::view::Handled::No
                }
            })
            .on_draft_commit(move |text| state.send(Event::Commit(text)));
        let toggle = button(picker_label.unwrap_or_else(|| message("time.open", "Choose time")))
            .node_ref(button_ref);
        let picked = binding(value, move |next| value.set(next));
        root.child(with_recipe_style(
            row((
                field,
                dialog(label, open)
                    .with_variants(&variants)
                    .mode(presentation.mode())
                    .return_focus(return_focus)
                    .hide_title(true)
                    .content(move || {
                        kivact_core::view::column((
                            crate::date_wheel::time_panel(picked, seconds),
                            button(message("time.save", "Save"))
                                .on_press(move || state.send(Event::Close)),
                        ))
                    })
                    .attach(toggle),
            )),
            |r| r.time_input_recipes().root_recipe(),
            &variants,
            PartState::default,
        ))
        .into_view()
    }
}

fn time_text(
    state: kivact_core::state::MachineHandle<Snapshot>,
    value: Binding<Option<TimeOfDay>>,
    seconds: Value<bool>,
    locale: Value<kivact_core::i18n::Locale>,
) -> Binding<String> {
    binding(
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
    )
}

fn time_controller(
    value: Binding<Option<TimeOfDay>>,
    seconds: Value<bool>,
    min: Value<Option<TimeOfDay>>,
    max: Value<Option<TimeOfDay>>,
) -> kivact_core::state::MachineHandle<Snapshot> {
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
    state
}
