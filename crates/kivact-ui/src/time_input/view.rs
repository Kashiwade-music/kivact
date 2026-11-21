//! Role: Time field and its wheel dialog.
use super::machine::{Event, Snapshot, TimeDraft, TimeInputState};
use crate::helpers::{is_named, report_unnamed};
use crate::layer::with_recipe_style;
use crate::time_input::look::TimeInputTheme;
use crate::{DatePresentation, button, dialog, text_input};
use kivact_core::custom::PartState;
use kivact_core::i18n::{TimeOfDay, TimeStyle, message, use_locale};
use kivact_core::state::{IntoText, IntoValue, Signal, Value, binding, machine};
use kivact_core::view::{Element, IntoView, Node, View, node, row};
use std::panic::Location;
// xtask-allow(state): record: the builder's props, each written only by its own setter; value is the caller's binding that the hour, minute and second fields write, and picker_label is the optional name of the picker button
/// A wall-clock time field with hours and minutes, optionally seconds.
pub struct TimeInput {
    root: Node,
    inherited: kivact_core::view::Variants,
    label: Option<Value<String>>,
    picker_label: Option<Value<String>>,
    value: Signal<Option<TimeOfDay>>,
    seconds: Value<bool>,
    min: Value<Option<TimeOfDay>>,
    max: Value<Option<TimeOfDay>>,
    presentation: DatePresentation,
    at: &'static Location<'static>,
}
/// Edit optional wall-clock time, independently of date and time zone.
/// Name it with [`label`](TimeInput::label).
#[track_caller]
pub fn time_input(value: Signal<Option<TimeOfDay>>) -> TimeInput {
    TimeInput {
        inherited: kivact_core::view::Variants::default(),
        root: node(),
        label: None,
        picker_label: None,
        value,
        seconds: false.into_value(),
        min: None.into_value(),
        max: None.into_value(),
        presentation: DatePresentation::Adaptive,
        at: Location::caller(),
    }
}
impl TimeInput {
    /// The field's accessible name, and the title of the wheel dialog.
    /// Inside a [`field`](crate::field) the field's label names it.
    #[track_caller]
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }

    /// Name the separate picker button independently of the text editor.
    #[track_caller]
    pub fn picker_label(mut self, label: impl IntoText) -> Self {
        self.picker_label = Some(label.into_text_value());
        self
    }

    pub(crate) fn with_variants(mut self, variants: &kivact_core::view::Variants) -> Self {
        self.inherited = variants.clone();
        self
    }
    /// Format the wall-clock time, read reactively.
    #[track_caller]
    pub fn format(mut self, format: impl IntoValue<TimeStyle>) -> Self {
        let format = format.into_value();
        self.seconds = (move || format.get().seconds).into_value();
        self
    }
    /// Earliest accepted wall-clock time.
    #[track_caller]
    pub fn min(mut self, min: impl IntoValue<Option<TimeOfDay>>) -> Self {
        self.min = min.into_value();
        self
    }
    /// Latest accepted wall-clock time.
    #[track_caller]
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
impl Element for TimeInput {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for TimeInput {
    fn into_view(self) -> View {
        let Self {
            mut root,
            label,
            picker_label,
            value,
            seconds,
            min,
            max,
            presentation,
            at,
            inherited,
        } = self;
        let variants = crate::layer::own_and_inherited(&root, &inherited);
        let named = is_named(&mut root) || label.is_some();
        report_unnamed(named, "time_input", at);
        let label = label.unwrap_or_else(|| String::new().into_value());
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
        let field = text_input(draft)
            .label(label)
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
            .on_commit(move |text| state.send(Event::Commit(text)));
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
                            crate::date_parts::wheel::time_panel(picked, seconds),
                            button(message("time.save", "Save"))
                                .on_press(move || state.send(Event::Close)),
                        ))
                    })
                    .attach(toggle),
            )),
            |r: &TimeInputTheme| r.root_recipe(),
            &variants,
            PartState::default,
        ))
        .into_view()
    }
}

fn time_text(
    state: kivact_core::state::MachineHandle<Snapshot>,
    value: Signal<Option<TimeOfDay>>,
    seconds: Value<bool>,
    locale: Value<kivact_core::i18n::Locale>,
) -> Signal<String> {
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
    value: Signal<Option<TimeOfDay>>,
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
    state.send(Event::External(value.get_untracked()));
    state.feed(value, Event::External);
    state
}
