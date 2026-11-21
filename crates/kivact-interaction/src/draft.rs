//! Role: Edit a draft of a value and write it to the value's signal on commit.
use kivact_core::state::{
    IntoValue, Machine as MachineTrait, MachineHandle, Signal, Step, Value, binding, effect_on,
    machine, memo,
};
use std::rc::Rc;

/// The value a draft stands for, `None` when it is not valid.
type Accept<T, D> = Rc<dyn Fn(&D) -> Option<T>>;

/// When an accepted draft is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftCommit {
    /// Every edit whose draft is accepted writes at once (a number field, a range's endpoints).
    Live,
    /// Only `commit()` and leaving the editor write (a page field).
    Explicit,
}

/// How a draft shows a value and which value a draft stands for.
pub struct DraftOptions<T: 'static, D: 'static> {
    show: Rc<dyn Fn(&T) -> D>,
    accept: Accept<T, D>,
    commit: DraftCommit,
    valid: Option<Signal<bool>>,
}

impl<T: 'static, D: 'static> DraftOptions<T, D> {
    /// `show` is the draft of a value (its text); `accept` is the value a draft
    /// stands for, `None` when it is not a valid value. Explicit commit.
    pub fn new(
        show: impl Fn(&T) -> D + 'static,
        accept: impl Fn(&D) -> Option<T> + 'static,
    ) -> Self {
        Self {
            show: Rc::new(show),
            accept: Rc::new(accept),
            commit: DraftCommit::Explicit,
            valid: None,
        }
    }

    pub fn commit(mut self, commit: DraftCommit) -> Self {
        self.commit = commit;
        self
    }

    /// Write whether the draft is accepted to `valid` after every change.
    pub fn valid(mut self, valid: Option<Signal<bool>>) -> Self {
        self.valid = valid;
        self
    }
}

#[derive(Clone, PartialEq)]
enum State<T, D> {
    /// The editor shows the value.
    Following,
    /// The user's draft; `produced` is the value its last accepted edit wrote (Live only).
    Editing { draft: D, produced: Option<T> },
}

enum Event<T, D> {
    Edit(D),
    Revise(D),
    Commit,
    Cancel,
    Blur,
    Changed(T),
    Wrote(T),
}

enum Effect<T> {
    Write(T),
    Valid(bool),
}

struct Machine<T: 'static, D: 'static> {
    value: T,
    show: Rc<dyn Fn(&T) -> D>,
    accept: Accept<T, D>,
    commit: DraftCommit,
}

impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> MachineTrait
    for Machine<T, D>
{
    type State = State<T, D>;
    type Event = Event<T, D>;
    type Effect = Effect<T>;
    fn transition(
        &self,
        state: &State<T, D>,
        event: Event<T, D>,
    ) -> Step<State<T, D>, Event<T, D>, Effect<T>> {
        let produced = match state {
            State::Editing { produced, .. } => produced.clone(),
            State::Following => None,
        };
        let next = match event {
            Event::Edit(draft) => {
                let accepted = (self.accept)(&draft);
                let step = self.to(State::Editing { draft, produced });
                return match accepted {
                    Some(value) if self.commit == DraftCommit::Live => {
                        step.with(Effect::Write(value))
                    }
                    _ => step,
                };
            }
            Event::Revise(draft) => State::Editing { draft, produced },
            Event::Wrote(value) => match state {
                State::Editing { draft, .. } => State::Editing {
                    draft: draft.clone(),
                    produced: Some(value),
                },
                State::Following => return Step::stay(),
            },
            Event::Commit => return self.commit_draft(state),
            Event::Blur if self.commit == DraftCommit::Explicit => return self.commit_draft(state),
            Event::Blur | Event::Cancel => State::Following,
            Event::Changed(value) => match state {
                State::Editing { draft, produced }
                    if (self.accept)(draft).as_ref() == Some(&value)
                        || produced.as_ref() == Some(&value) =>
                {
                    return Step::stay();
                }
                _ => State::Following,
            },
        };
        self.to(next)
    }
}

impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> Machine<T, D> {
    /// Go to `next` and report whether its draft is accepted.
    fn to(&self, next: State<T, D>) -> Step<State<T, D>, Event<T, D>, Effect<T>> {
        let valid = match &next {
            State::Following => (self.accept)(&(self.show)(&self.value)).is_some(),
            State::Editing { draft, .. } => (self.accept)(draft).is_some(),
        };
        Step::to(next).with(Effect::Valid(valid))
    }

    fn commit_draft(&self, state: &State<T, D>) -> Step<State<T, D>, Event<T, D>, Effect<T>> {
        let State::Editing { draft, .. } = state else {
            return Step::stay();
        };
        match (self.accept)(draft) {
            Some(value) if self.commit == DraftCommit::Explicit => {
                self.to(State::Following).with(Effect::Write(value))
            }
            Some(_) => self.to(State::Following),
            None => Step::stay(),
        }
    }
}

/// A draft of `value` that an editor changes and a commit writes.
pub struct DraftApi<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> {
    session: MachineHandle<Machine<T, D>>,
    draft: Value<D>,
    editing: Value<bool>,
    valid: Value<bool>,
}

impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> Clone for DraftApi<T, D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> Copy for DraftApi<T, D> {}

/// Create a draft of `value`. Setup only.
#[track_caller]
pub fn use_draft<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static>(
    value: Signal<T>,
    options: DraftOptions<T, D>,
) -> DraftApi<T, D> {
    let DraftOptions {
        show,
        accept,
        commit,
        valid,
    } = options;
    let (snapshot_show, snapshot_accept) = (show.clone(), accept.clone());
    let session = machine(
        State::Following,
        move || Machine {
            value: value.get_untracked(),
            show: snapshot_show.clone(),
            accept: snapshot_accept.clone(),
            commit,
        },
        move |cx, effect| match effect {
            Effect::Write(next) => {
                value.set(next);
                cx.send(Event::Wrote(value.get_untracked()));
            }
            Effect::Valid(next) => {
                if let Some(valid) = valid {
                    valid.set(next);
                }
            }
        },
    );
    session.feed(value, Event::Changed);
    let draft_show = show.clone();
    let draft = memo(move || match session.get() {
        State::Following => draft_show(&value.get()),
        State::Editing { draft, .. } => draft,
    });
    let editing = memo(move || matches!(session.get(), State::Editing { .. }));
    let valid = memo(move || draft.with(|draft| accept(draft).is_some()));
    DraftApi {
        session,
        draft,
        editing,
        valid,
    }
}

impl<T: Clone + PartialEq + 'static, D: Clone + PartialEq + 'static> DraftApi<T, D> {
    /// The draft the editor shows: the value's while following it.
    pub fn draft(self) -> Value<D> {
        self.draft
    }

    pub fn is_editing(self) -> Value<bool> {
        self.editing
    }

    /// Whether the shown draft stands for a value.
    pub fn is_valid(self) -> Value<bool> {
        self.valid
    }

    /// Replace the draft (Live: an accepted draft is written at once).
    pub fn edit(self, draft: D) {
        self.session.send(Event::Edit(draft));
    }

    /// Change the shown draft in place, as `edit` does.
    pub fn update(self, change: impl FnOnce(&mut D)) {
        let mut draft = self.draft.get_untracked();
        change(&mut draft);
        self.edit(draft);
    }

    /// Change the shown draft without writing, even when Live (a validity report).
    pub fn revise(self, change: impl FnOnce(&mut D)) {
        let mut draft = self.draft.get_untracked();
        change(&mut draft);
        self.session.send(Event::Revise(draft));
    }

    /// Write an accepted draft (Explicit) and follow the value again; a draft that is not accepted stays.
    pub fn commit(self) {
        self.session.send(Event::Commit);
    }

    /// Follow the value again without writing.
    pub fn cancel(self) {
        self.session.send(Event::Cancel);
    }

    /// Leaving the editor (`focused` turns false) commits (Explicit) or follows the value (Live). Setup only.
    #[track_caller]
    pub fn settle_on_blur(self, focused: impl IntoValue<bool>) {
        let session = self.session;
        effect_on(focused.into_value(), move |focused| {
            if !*focused {
                session.send(Event::Blur);
            }
        });
    }

    /// The draft as a signal an editor edits. Setup only.
    #[track_caller]
    pub fn editor(self) -> Signal<D> {
        binding(self.draft, move |draft| self.edit(draft))
    }
}
