//! Role: Decide active-row reveal and apply ancestor expansion after transitions.
use super::controls::Rows;
use crate::TreeItem;
use kivact::state::{Binding, Machine, MachineHandle, Step, Value, machine};
use kivact::view::NodeRef;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum State {
    Idle,
    AwaitingRow(u64),
}
pub(super) enum Event {
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
            Event::Active {
                target,
                ancestors,
                node,
            } => {
                let next = match (target, node) {
                    (Some(id), None) => State::AwaitingRow(id),
                    _ => State::Idle,
                };
                let mut step = Step::to(next);
                if !ancestors.is_empty() {
                    step = step.with(Effect::Expand(ancestors));
                }
                if let Some(node) = node {
                    step = step.with(Effect::Scroll(node));
                }
                step
            }
            Event::Mounted { id, node } if *state == State::AwaitingRow(id) => {
                Step::to(State::Idle).with(Effect::Scroll(node))
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
    pressed: Rc<Cell<Option<u64>>>,
) -> MachineHandle<Reveal> {
    let reveal = machine(
        State::Idle,
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
        let target = target.filter(|id| pressed.get() != Some(*id));
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
