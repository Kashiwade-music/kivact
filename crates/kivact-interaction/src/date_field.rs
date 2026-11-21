//! Role: A date or time field: its typed text, its checks and its picker.
mod machine;

/// Reads typed text as a value, `None` when it is not one.
pub(super) type Parse<T> = Rc<dyn Fn(&str) -> Option<T>>;

use kivact_core::state::{
    Callback, IntoValue, MachineHandle, Signal, Value, binding, callback, machine, memo, signal,
};
use kivact_core::view::{HandlerResult, NodeRef};
use std::rc::Rc;

/// Which panel of a picker shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePanel {
    Date,
    Time,
}

/// A picker's state: closed, or open on a panel, opened from `invoker`.
#[derive(Clone, Copy, PartialEq)]
pub enum DatePicker {
    Closed,
    Open { panel: DatePanel, invoker: NodeRef },
}

/// The typed text of a date or time field.
#[derive(Clone, Debug, PartialEq)]
pub enum TypedText {
    /// The field shows the value, formatted by the caller.
    Following,
    /// The user's text and whether a commit rejected it.
    Editing { text: String, check: TextCheck },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextCheck {
    Unchecked,
    Rejected,
}

/// How typed text that is no valid value shows and closes the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedCheck {
    /// Invalid only after a commit rejected it; a rejected commit keeps the picker (a date field).
    OnCommit,
    /// Invalid as soon as it is no valid value; every commit closes the picker (a time field).
    WhileTyping,
}

pub struct DateFieldOptions<T: 'static> {
    parse: Parse<T>,
    accept: Rc<dyn Fn(T) -> bool>,
    check: TypedCheck,
    valid: Option<Signal<bool>>,
    on_write: Option<Callback<Option<T>>>,
}

impl<T: 'static> DateFieldOptions<T> {
    /// `parse` reads typed text; `accept` is the bounds. `TypedCheck::OnCommit`.
    pub fn new(
        parse: impl Fn(&str) -> Option<T> + 'static,
        accept: impl Fn(T) -> bool + 'static,
    ) -> Self {
        Self {
            parse: Rc::new(parse),
            accept: Rc::new(accept),
            check: TypedCheck::OnCommit,
            valid: None,
            on_write: None,
        }
    }

    pub fn check(mut self, check: TypedCheck) -> Self {
        self.check = check;
        self
    }

    /// Write whether the typed text is a valid optional value after every change.
    pub fn valid(mut self, valid: Option<Signal<bool>>) -> Self {
        self.valid = valid;
        self
    }

    /// Call `f` with each value the field writes (not with external writes). Setup only.
    #[track_caller]
    pub fn on_write<R: HandlerResult>(mut self, f: impl Fn(Option<T>) -> R + 'static) -> Self {
        self.on_write = Some(callback(f));
        self
    }
}

#[derive(Clone, Copy)]
pub struct DatePickerApi {
    state: Signal<DatePicker>,
    open: Value<bool>,
}

/// A picker's open state, its panel and where the focus returns. Setup only.
#[track_caller]
pub fn use_date_picker() -> DatePickerApi {
    let state = signal(DatePicker::Closed);
    DatePickerApi {
        state,
        open: memo(move || matches!(state.get(), DatePicker::Open { .. })),
    }
}

impl DatePickerApi {
    pub fn show(self, panel: DatePanel, invoker: NodeRef) {
        self.state.set(DatePicker::Open { panel, invoker });
    }

    /// Show `panel` while open; nothing while closed.
    pub fn show_panel(self, panel: DatePanel) {
        if let DatePicker::Open { invoker, .. } = self.state.get_untracked() {
            self.state.set(DatePicker::Open { panel, invoker });
        }
    }

    pub fn hide(self) {
        self.state.set(DatePicker::Closed);
    }

    pub fn state(self) -> Value<DatePicker> {
        self.state.into_value()
    }

    pub fn is_open(self) -> Value<bool> {
        self.open
    }

    /// The open state as a signal: opening shows `panel` from `invoker`. Setup only.
    #[track_caller]
    pub fn open(self, panel: DatePanel, invoker: NodeRef) -> Signal<bool> {
        binding(self.open, move |open| {
            if open {
                self.show(panel, invoker);
            } else {
                self.hide();
            }
        })
    }

    /// Where the focus returns on close: the invoker, else `fallback`. Setup only.
    #[track_caller]
    pub fn return_focus(self, fallback: NodeRef) -> Value<NodeRef> {
        let state = self.state;
        memo(move || match state.get() {
            DatePicker::Open { invoker, .. } => invoker,
            DatePicker::Closed => fallback,
        })
    }
}

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
