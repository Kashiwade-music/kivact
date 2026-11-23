//! Role: The date field handle: its typed text, its picker and its commands.

use kivact_core::state::{MachineHandle, Signal, Value, binding, machine, memo};

use super::machine;
use super::options::DateFieldOptions;
use super::picker::{DatePickerApi, use_date_picker};
use super::types::{TextCheck, TypedCheck, TypedText};

pub struct DateFieldApi<T: Copy + PartialEq + 'static> {
    value: Signal<Option<T>>,
    session: MachineHandle<machine::Machine<T>>,
    picker: DatePickerApi,
    typed: Value<TypedText>,
    invalid: Value<bool>,
}

impl<T: Copy + PartialEq + 'static> Clone for DateFieldApi<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy + PartialEq + 'static> Copy for DateFieldApi<T> {}

/// Create the typed text and the picker of a field editing `value`. Setup only.
#[track_caller]
pub fn use_date_field<T: Copy + PartialEq + 'static>(
    value: Signal<Option<T>>,
    options: DateFieldOptions<T>,
) -> DateFieldApi<T> {
    let DateFieldOptions {
        parse,
        accept,
        check,
        valid,
        on_write,
    } = options;
    let picker = use_date_picker();
    // The rules the `invalid` memo reads; the machine builds its own snapshot per event.
    let rules = machine::Machine {
        parse: parse.clone(),
        accept: accept.clone(),
        check,
    };
    let session = machine(
        TypedText::Following,
        move || machine::Machine {
            parse: parse.clone(),
            accept: accept.clone(),
            check,
        },
        move |_, effect| match effect {
            machine::Effect::Write(next) => {
                value.set(next);
                if let Some(on_write) = on_write {
                    on_write.call(next);
                }
            }
            machine::Effect::ClosePicker => picker.hide(),
            machine::Effect::Valid(next) => {
                if let Some(valid) = valid {
                    valid.set(next);
                }
            }
        },
    );
    session.feed(value, |_| machine::Event::External);
    let typed = session.state();
    let invalid = memo(move || match typed.get() {
        TypedText::Following => false,
        TypedText::Editing { text, check: state } => {
            !text.trim().is_empty()
                && match rules.check {
                    TypedCheck::OnCommit => state == TextCheck::Rejected,
                    TypedCheck::WhileTyping => rules.accepted(&text).is_none(),
                }
        }
    });
    DateFieldApi {
        value,
        session,
        picker,
        typed,
        invalid,
    }
}

impl<T: Copy + PartialEq + 'static> DateFieldApi<T> {
    pub fn typed(self) -> Value<TypedText> {
        self.typed
    }

    /// Whether the field shows its typed text as invalid.
    pub fn is_invalid(self) -> Value<bool> {
        self.invalid
    }

    pub fn edit(self, text: String) {
        self.session.send(machine::Event::Edit(text));
    }

    /// Commit typed text: empty writes `None`, a valid value writes it, other text is rejected.
    pub fn commit(self, text: String) {
        self.session.send(machine::Event::Commit(text));
    }

    /// Write a picked value within the bounds and close the picker.
    pub fn pick(self, value: T) {
        self.session.send(machine::Event::Pick(value));
    }

    /// Write a picked value within the bounds and keep the picker open (a sheet's wheel).
    pub fn pick_in_place(self, value: T) {
        self.session.send(machine::Event::PickInPlace(value));
    }

    pub fn clear(self) {
        self.session.send(machine::Event::Clear);
    }

    pub fn picker(self) -> DatePickerApi {
        self.picker
    }

    /// The text the editor shows: `format` of the value while following it. Setup only.
    #[track_caller]
    pub fn editor(self, format: impl Fn(T) -> String + 'static) -> Signal<String> {
        let (typed, value) = (self.typed, self.value);
        binding(
            move || match typed.get() {
                TypedText::Following => value.get().map(&format).unwrap_or_default(),
                TypedText::Editing { text, .. } => text,
            },
            move |text| self.edit(text),
        )
    }
}
