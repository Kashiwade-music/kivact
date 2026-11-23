//! Role: The typed text of a date or time field: follows the value or is edited.
//!
//! It is checked on commit.
use super::types::{Parse, TextCheck, TypedCheck, TypedText};
use kivact_core::state::{Machine as MachineTrait, Step};
use std::rc::Rc;

pub(super) enum Event<T> {
    External,
    Edit(String),
    Commit(String),
    Pick(T),
    PickInPlace(T),
    Clear,
}

pub(super) enum Effect<T> {
    Write(Option<T>),
    ClosePicker,
    Valid(bool),
}

pub(super) struct Machine<T: 'static> {
    pub(super) parse: Parse<T>,
    pub(super) accept: Rc<dyn Fn(T) -> bool>,
    pub(super) check: TypedCheck,
}

impl<T: Copy + 'static> Machine<T> {
    pub(super) fn accepted(&self, text: &str) -> Option<T> {
        (self.parse)(text).filter(|value| (self.accept)(*value))
    }

    fn valid(&self, state: &TypedText) -> bool {
        match state {
            TypedText::Following => true,
            TypedText::Editing { text, .. } => {
                text.trim().is_empty() || self.accepted(text).is_some()
            }
        }
    }

    fn to(&self, next: TypedText) -> Step<TypedText, Event<T>, Effect<T>> {
        let valid = self.valid(&next);
        Step::to(next).with(Effect::Valid(valid))
    }
}

impl<T: Copy + 'static> MachineTrait for Machine<T> {
    type State = TypedText;
    type Event = Event<T>;
    type Effect = Effect<T>;
    fn transition(&self, _: &TypedText, event: Event<T>) -> Step<TypedText, Event<T>, Effect<T>> {
        match event {
            Event::External => self.to(TypedText::Following),
            Event::Edit(text) => self.to(TypedText::Editing {
                text,
                check: TextCheck::Unchecked,
            }),
            Event::Commit(text) if text.trim().is_empty() => self
                .to(TypedText::Following)
                .with(Effect::Write(None))
                .with(Effect::ClosePicker),
            Event::Commit(text) => match self.accepted(&text) {
                Some(value) => self
                    .to(TypedText::Following)
                    .with(Effect::Write(Some(value)))
                    .with(Effect::ClosePicker),
                None => {
                    let step = self.to(TypedText::Editing {
                        text,
                        check: TextCheck::Rejected,
                    });
                    if self.check == TypedCheck::WhileTyping {
                        step.with(Effect::ClosePicker)
                    } else {
                        step
                    }
                }
            },
            Event::Pick(value) if (self.accept)(value) => self
                .to(TypedText::Following)
                .with(Effect::Write(Some(value)))
                .with(Effect::ClosePicker),
            Event::PickInPlace(value) if (self.accept)(value) => self
                .to(TypedText::Following)
                .with(Effect::Write(Some(value))),
            Event::Pick(_) | Event::PickInPlace(_) => Step::stay(),
            Event::Clear => self
                .to(TypedText::Following)
                .with(Effect::Write(None))
                .with(Effect::ClosePicker),
        }
    }
}
