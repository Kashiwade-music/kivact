//! Role: Decide active-row reveal and apply ancestor expansion after transitions.
use super::controls::Rows;
use crate::TreeItem;
use kivact::state::{Binding, Machine, MachineHandle, Step, Value, machine};
use kivact::view::NodeRef;

/// Whether a reveal waits for its row to mount.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Pending {
    Idle,
    AwaitingRow(u64),
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct State {
    /// The row a primary press is on, from its press to its release; a
    /// press activates its own row without scrolling it.
    pressed: Option<u64>,
    pending: Pending,
}
pub(super) enum Event {
    Press(Option<u64>),
    Active {
        target: Option<u64>,
        ancestors: Vec<u64>,
        node: Option<NodeRef>,
    },
    Mounted {
        id: u64,
        node: NodeRef,
    },
}
pub(super) enum Effect {
    Expand(Vec<u64>),
    Scroll(NodeRef),
}
pub(super) struct Reveal;
impl Machine for Reveal {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match event {
            Event::Press(pressed) => Step::to(State { pressed, ..*state }),
            Event::Active {
                target,
                ancestors,
                node,
            } => {
                if target.is_some() && target == state.pressed {
                    return Step::to(State {
                        pending: Pending::Idle,
                        ..*state
                    });
                }
                let pending = match (target, node) {
                    (Some(id), None) => Pending::AwaitingRow(id),
                    _ => Pending::Idle,
                };
                let mut step = Step::to(State { pending, ..*state });
                if !ancestors.is_empty() {
                    step = step.with(Effect::Expand(ancestors));
                }
                if let Some(node) = node {
                    step = step.with(Effect::Scroll(node));
                }
                step
            }
            Event::Mounted { id, node } if state.pending == Pending::AwaitingRow(id) => {
                Step::to(State {
                    pending: Pending::Idle,
                    ..*state
                })
                .with(Effect::Scroll(node))
            }
            Event::Mounted { .. } => Step::stay(),
        }
    }
}
pub(super) fn observe(
    active: Value<Option<u64>>,
    enabled: Value<bool>,
    data: Value<Vec<TreeItem>>,
    expanded: Binding<Vec<u64>>,
    rows: Rows,
) -> MachineHandle<Reveal> {
    let reveal = machine(
        State {
            pressed: None,
            pending: Pending::Idle,
        },
        || Reveal,
        move |_, effect| match effect {
            Effect::Expand(ancestors) => {
                let mut open = expanded.get_untracked();
                open.extend(ancestors);
                expanded.set(open);
            }
            Effect::Scroll(node) => node.scroll_into_view(),
        },
    );
    kivact::state::effect_on(active, move |&target| {
        if !enabled.get_untracked() {
            return;
        }
        let ancestors = target.map_or_else(Vec::new, |id| {
            closed_ancestors(id, &data.get_untracked(), &expanded.get_untracked())
        });
        let node = target.and_then(|id| rows.borrow().get(&id).map(|row| row.node));
        reveal.send(Event::Active {
            target,
            ancestors,
            node,
        });
    });
    reveal
}
fn closed_ancestors(id: u64, items: &[TreeItem], expanded: &[u64]) -> Vec<u64> {
    let mut closed = Vec::new();
    let mut parent = items.iter().find(|n| n.id == id).and_then(|n| n.parent);
    for _ in 0..items.len() {
        let Some(ancestor) = parent.and_then(|p| items.iter().find(|n| n.id == p)) else {
            break;
        };
        if !expanded.contains(&ancestor.id) {
            closed.push(ancestor.id);
        }
        parent = ancestor.parent;
    }
    closed
}
