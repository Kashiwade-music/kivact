//! Role: Recall submitted entries in a text editor with Up and Down.
use kivact_core::custom::{EditState, NodeRefExt, Part, PartState, part};
use kivact_core::state::{IntoValue, Machine as MachineTrait, MachineHandle, Step, Value, machine};
use kivact_core::view::{Handled, Key, KeyEvent, Modifiers, NodeRef};

#[derive(Clone, PartialEq)]
enum Recall {
    Draft,
    Entry { index: usize, draft: String },
}

#[derive(Clone, PartialEq)]
struct History {
    entries: Vec<String>,
    recall: Recall,
}

enum Event {
    Submit(String),
    Recall { previous: bool, draft: String },
}

struct Machine;

impl MachineTrait for Machine {
    type State = History;
    type Event = Event;
    type Effect = String;
    fn transition(&self, state: &History, event: Event) -> Step<History, Event, String> {
        let mut next = state.clone();
        match event {
            Event::Submit(value) => {
                next.entries.push(value);
                next.recall = Recall::Draft;
                Step::to(next)
            }
            Event::Recall { previous, draft } => {
                let (index, saved) = match &state.recall {
                    Recall::Draft if previous && !state.entries.is_empty() => {
                        (state.entries.len(), draft)
                    }
                    Recall::Entry { index, draft } => (*index, draft.clone()),
                    _ => return Step::stay(),
                };
                let index = if previous {
                    index.saturating_sub(1)
                } else {
                    index + 1
                };
                let value = match state.entries.get(index) {
                    Some(value) => {
                        next.recall = Recall::Entry {
                            index,
                            draft: saved,
                        };
                        value.clone()
                    }
                    None => {
                        next.recall = Recall::Draft;
                        saved
                    }
                };
                Step::to(next).with(value)
            }
        }
    }
}

/// Submitted entries an editor recalls with Up and Down.
#[derive(Clone, Copy)]
pub struct HistoryApi {
    session: MachineHandle<Machine>,
    editor: NodeRef,
    enabled: Value<bool>,
}

/// The history of the text editor `editor`, active while `enabled`. Setup only.
#[track_caller]
pub fn use_history(editor: NodeRef, enabled: impl IntoValue<bool>) -> HistoryApi {
    let session = machine(
        History {
            entries: Vec::new(),
            recall: Recall::Draft,
        },
        || Machine,
        move |_, value: String| {
            let mut state = EditState::new(&value);
            state.move_to(0, false);
            state.move_to(value.len(), true);
            editor.set_edit_state(state);
        },
    );
    HistoryApi {
        session,
        editor,
        enabled: enabled.into_value(),
    }
}

impl HistoryApi {
    /// Record a submitted entry (while enabled) and go back to the draft.
    pub fn push(self, entry: impl Into<String>) {
        if self.enabled.get_untracked() {
            self.session.send(Event::Submit(entry.into()));
        }
    }

    /// The editor's part: a plain Up with the caret at the start recalls the
    /// previous entry, a plain Down at the end the next one or the draft. Setup only.
    #[track_caller]
    pub fn input(self) -> Part<PartState> {
        let input = part(|input| input);
        Part::new(input.state(), move |host| {
            host.part(input)
                .on_key(move |event: KeyEvent| self.key(&event))
        })
    }

    fn key(self, event: &KeyEvent) -> Handled {
        if !self.enabled.get_untracked()
            || event.modifiers() != Modifiers::default()
            || !matches!(event.key(), Key::ArrowUp | Key::ArrowDown)
        {
            return Handled::No;
        }
        let Some(editor) = self.editor.edit_state() else {
            return Handled::No;
        };
        let selection = editor.selection();
        let at_start = selection.start == 0;
        let at_end = selection.end == editor.value().len();
        let eligible = match event.key() {
            Key::ArrowUp => at_start && (selection.is_empty() || at_end),
            Key::ArrowDown => at_end && (selection.is_empty() || at_start),
            _ => false,
        };
        if !eligible {
            return Handled::No;
        }
        self.session.send(Event::Recall {
            previous: event.key() == Key::ArrowUp,
            draft: editor.value().to_owned(),
        });
        Handled::Yes
    }
}
