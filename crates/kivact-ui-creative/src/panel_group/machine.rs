//! Role: Decide edit state transitions.
use super::data::InspectorPanel;
use kivact::state::Step;
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum State {
    Idle,
    Dragging {
        panel: u64,
        target: Option<(u64, bool)>,
    },
}
impl State {
    pub(crate) fn panel(self) -> Option<u64> {
        match self {
            Self::Idle => None,
            Self::Dragging { panel, .. } => Some(panel),
        }
    }
    pub(crate) fn target(self) -> Option<(u64, bool)> {
        match self {
            Self::Idle => None,
            Self::Dragging { target, .. } => target,
        }
    }
}
pub(crate) struct Machine {
    pub(crate) panels: Vec<InspectorPanel>,
}
pub(crate) enum Event {
    Begin(u64),
    Target(Option<(u64, bool)>),
    End(bool),
    Removed(u64),
}
impl kivact::state::Machine for Machine {
    type State = State;
    type Event = Event;
    type Effect = Vec<InspectorPanel>;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Self::Effect> {
        match (*state, event) {
            (_, Event::Begin(panel)) => Step::to(State::Dragging {
                panel,
                target: None,
            }),
            (State::Dragging { panel, .. }, Event::Target(target)) => {
                Step::to(State::Dragging { panel, target })
            }
            (
                State::Dragging {
                    panel,
                    target: Some((destination, after)),
                },
                Event::End(true),
            ) => {
                let mut next = self.panels.clone();
                if let Some(index) = next.iter().position(|item| item.id == panel) {
                    let item = next.remove(index);
                    if let Some(index) = next.iter().position(|item| item.id == destination) {
                        next.insert(index + usize::from(after), item);
                        return Step::to(State::Idle).with(next);
                    }
                }
                Step::to(State::Idle)
            }
            (_, Event::End(_)) => Step::to(State::Idle),
            (State::Dragging { panel, .. }, Event::Removed(id)) if panel == id => {
                Step::to(State::Idle)
            }
            _ => Step::stay(),
        }
    }
}
