//! Role: Own submitted drafts and recall transitions without duplicating editor state.
use kivact_core::custom::{EditState, NodeRefExt};
use kivact_core::state::{Machine, MachineHandle, Step, machine};
use kivact_core::view::NodeRef;
#[derive(Clone, PartialEq)]
enum Recall {
    Draft,
    Entry { index: usize, draft: String },
}
#[derive(Clone, PartialEq)]
pub(super) struct History {
    entries: Vec<String>,
    recall: Recall,
}
pub(super) enum Event {
    Submit(String),
    Recall { previous: bool, draft: String },
}
pub(super) struct Policy;
impl Machine for Policy {
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
pub(super) fn use_history(input: NodeRef) -> MachineHandle<Policy> {
    machine(
        History {
            entries: Vec::new(),
            recall: Recall::Draft,
        },
        || Policy,
        move |_, value: String| {
            let mut editor = EditState::new(&value);
            editor.move_to(value.len(), true);
            input.set_edit_state(editor);
        },
    )
}
