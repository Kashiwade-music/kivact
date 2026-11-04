//! Role: Store the held pointer, surfaces and row highlights of a menu.
use super::press::machine::{Event, Highlight, Machine, PressGesture, State};
use kivact_core::geometry::Point;
use kivact_core::state::{MachineHandle, Value, Var, machine, on_cleanup, var};
use kivact_core::view::NodeRef;

pub use crate::menu_policy::MenuPress;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Source {
    Trigger,
    Menu,
    Context,
}

#[derive(Clone, Copy)]
struct Row {
    reference: NodeRef,
    disabled: Value<bool>,
}

#[derive(Clone, Copy)]
pub(super) struct Press {
    pub(super) machine: MachineHandle<Machine>,
    surfaces: Var<Vec<NodeRef>>,
    rows: Var<Vec<Row>>,
}

impl Press {
    pub(super) fn new() -> Self {
        Self {
            machine: machine(
                State {
                    gesture: PressGesture::Idle,
                    highlight: Highlight::Unset,
                },
                || Machine {
                    now: kivact_core::custom::clock_now(),
                },
                |_, callback| callback.call(()),
            ),
            surfaces: var(Vec::new()),
            rows: var(Vec::new()),
        }
    }

    pub(super) fn surface(self, node: NodeRef) {
        self.surfaces.update(|surfaces| surfaces.push(node));
        on_cleanup(move || {
            self.surfaces
                .update(|surfaces| surfaces.retain(|item| *item != node))
        });
    }

    pub(super) fn row(self, reference: NodeRef, disabled: Value<bool>) {
        self.rows.update(|rows| {
            rows.push(Row {
                reference,
                disabled,
            })
        });
        on_cleanup(move || {
            self.rows
                .update(|rows| rows.retain(|row| row.reference != reference))
        });
    }

    pub(super) fn highlighted(self, row: NodeRef) -> Option<bool> {
        match self.machine.get().highlight {
            Highlight::Unset => None,
            Highlight::Nothing => Some(false),
            Highlight::Item(active) => Some(active == row),
        }
    }

    pub(super) fn hit(self, position: Point) -> (bool, Option<NodeRef>) {
        let inside = self
            .surfaces
            .with(|surfaces| surfaces.iter().any(|node| node.hit_test(position)));
        let row = inside
            .then(|| {
                self.rows.with(|rows| {
                    rows.iter()
                        .rev()
                        .find(|row| {
                            !row.disabled.get_untracked() && row.reference.hit_test(position)
                        })
                        .map(|row| row.reference)
                })
            })
            .flatten();
        (inside, row)
    }

    pub(super) fn cancel_open(self) {
        if matches!(
            self.machine.get().gesture,
            PressGesture::Holding { .. } | PressGesture::Dragging { .. }
        ) {
            self.cancel();
        }
    }
    pub(super) fn cancel(self) {
        self.machine.send(Event::Cancel);
    }
}
