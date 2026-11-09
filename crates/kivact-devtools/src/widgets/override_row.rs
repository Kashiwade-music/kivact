//! Role: One override row: a checkbox toggles it; the editor edits its value.

use kivact::prelude::*;
use kivact::state::{Machine, Step, machine};

use crate::widgets::property_row::{RowAccess, color_property_row, property_row};

/// An override row's state: off with the value to use when it is turned on, or on with the
/// agent's id (when the agent had an override for the property at build time).
#[derive(Clone, PartialEq)]
pub(crate) enum Override {
    /// No override; `value` is what turning it on writes.
    Off { value: String },
    /// An override; `id` is the agent's override, if it had one.
    On { id: Option<u64>, value: String },
}

impl Override {
    /// The value the row shows.
    fn value(&self) -> &str {
        match self {
            Override::Off { value } | Override::On { value, .. } => value,
        }
    }
}

/// The events of an override row.
enum OverrideEvent {
    /// The checkbox changed.
    Toggle(bool),
    /// The editor's text changed.
    Edit(String),
    /// The editor submitted its text.
    Commit,
}

/// What an override row asks its panel to do.
enum OverrideEffect {
    /// Write the override with this value.
    Set(String),
    /// Remove the override of this id (nothing when there is no id yet).
    Clear(Option<u64>),
}

/// The machine of an override row; its state is the row's state.
struct OverrideMachine;

impl Machine for OverrideMachine {
    type State = Override;
    type Event = OverrideEvent;
    type Effect = OverrideEffect;

    fn transition(
        &self,
        state: &Override,
        event: OverrideEvent,
    ) -> Step<Self::State, Self::Event, Self::Effect> {
        match event {
            OverrideEvent::Toggle(true) => match state {
                Override::On { .. } => Step::stay(),
                Override::Off { value } => Step::to(Override::On {
                    id: None,
                    value: value.clone(),
                })
                .with(OverrideEffect::Set(value.clone())),
            },
            OverrideEvent::Toggle(false) => match state {
                Override::Off { .. } => Step::stay(),
                Override::On { id, value } => Step::to(Override::Off {
                    value: value.clone(),
                })
                .with(OverrideEffect::Clear(*id)),
            },
            OverrideEvent::Edit(text) => Step::to(match state {
                Override::Off { .. } => Override::Off { value: text },
                Override::On { id, .. } => Override::On {
                    id: *id,
                    value: text,
                },
            }),
            OverrideEvent::Commit => match state {
                Override::Off { .. } => Step::stay(),
                Override::On { value, .. } => Step::stay().with(OverrideEffect::Set(value.clone())),
            },
        }
    }
}

/// One editable property row with an override checkbox; `color` picks the color editor.
pub(crate) fn override_row(
    name: String,
    initial: Override,
    color: bool,
    access: RowAccess,
    on_set: impl Fn(String) + 'static,
    on_clear: impl Fn(u64) + 'static,
) -> View {
    let over = machine(
        initial,
        || OverrideMachine,
        move |_, effect| match effect {
            OverrideEffect::Set(value) => on_set(value),
            OverrideEffect::Clear(id) => {
                if let Some(id) = id {
                    on_clear(id);
                }
            }
        },
    );
    let enabled = binding(
        move || matches!(over.get(), Override::On { .. }),
        move |on: bool| over.send(OverrideEvent::Toggle(on)),
    );
    let value = binding(
        move || over.get().value().to_owned(),
        move |text: String| over.send(OverrideEvent::Edit(text)),
    );
    let commit = move |_: String| over.send(OverrideEvent::Commit);
    if color {
        color_property_row(name, value, enabled, access, commit).into_view()
    } else {
        property_row(name, value, enabled, access, commit).into_view()
    }
}
