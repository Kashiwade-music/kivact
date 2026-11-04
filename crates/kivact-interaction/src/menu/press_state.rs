//! Role: Store the held pointer, surfaces and row highlights of a menu.
use kivact_core::geometry::Point;
use kivact_core::state::{
    Callback, Machine, MachineHandle, Step, Value, Var, machine, on_cleanup, var,
};
use kivact_core::view::{NodeRef, PointerId, PointerKind};

pub use crate::menu_policy::MenuPress;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Source {
    Trigger,
    Menu,
    Context,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum PressGesture {
    Idle,
    Pending {
        from: Point,
        source: PointerId,
        input: Input,
    },
    Holding {
        at: Point,
        since: u64,
        input: Input,
    },
    Dragging {
        at: Point,
        since: u64,
        input: Input,
    },
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Input {
    pub(super) id: PointerId,
    pub(super) kind: PointerKind,
    pub(super) source: Source,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Highlight {
    Unset,
    Nothing,
    Item(NodeRef),
}
impl From<Option<NodeRef>> for Highlight {
    fn from(row: Option<NodeRef>) -> Self {
        row.map_or(Self::Nothing, Self::Item)
    }
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct PressState {
    pub(super) gesture: PressGesture,
    highlight: Highlight,
}
pub(super) enum PressEvent {
    Pending(Input, Point, Callback<()>),
    Hold(Input, Point, Callback<()>),
    Open(Input, Point, u64, bool),
    Move(Point, Highlight),
    Highlight(Highlight),
    Cancel,
}
pub(super) struct PressMachine {
    now: u64,
}
impl Machine for PressMachine {
    type State = PressState;
    type Event = PressEvent;
    type Effect = Callback<()>;
    fn transition(
        &self,
        state: &PressState,
        event: PressEvent,
    ) -> Step<PressState, PressEvent, Callback<()>> {
        let mut next = *state;
        match event {
            PressEvent::Pending(input, from, callback) => {
                next.gesture = PressGesture::Pending {
                    from,
                    source: input.id,
                    input,
                };
                return Step::to(next).after(
                    std::time::Duration::from_millis(500),
                    PressEvent::Hold(input, from, callback),
                );
            }
            PressEvent::Hold(input, at, callback) => {
                if !matches!(state.gesture, PressGesture::Pending { source, .. } if source == input.id)
                {
                    return Step::stay();
                }
                next.gesture = PressGesture::Holding {
                    at,
                    since: self.now,
                    input,
                };
                return Step::to(next).with(callback);
            }
            PressEvent::Open(input, at, since, entered) => {
                next.gesture = if entered {
                    PressGesture::Dragging { at, since, input }
                } else {
                    PressGesture::Holding { at, since, input }
                };
            }
            PressEvent::Move(at, highlight) => {
                match state.gesture {
                    PressGesture::Holding { since, input, .. }
                    | PressGesture::Dragging { since, input, .. } => {
                        next.gesture = PressGesture::Dragging { at, since, input }
                    }
                    _ => return Step::stay(),
                }
                next.highlight = highlight;
            }
            PressEvent::Highlight(highlight) => next.highlight = highlight,
            PressEvent::Cancel => {
                next.gesture = PressGesture::Idle;
                next.highlight = Highlight::Unset;
            }
        }
        Step::to(next)
    }
}

#[derive(Clone, Copy)]
struct Row {
    reference: NodeRef,
    disabled: Value<bool>,
}

#[derive(Clone, Copy)]
pub(super) struct Press {
    pub(super) machine: MachineHandle<PressMachine>,
    surfaces: Var<Vec<NodeRef>>,
    rows: Var<Vec<Row>>,
}

impl Press {
    pub(super) fn new() -> Self {
        Self {
            machine: machine(
                PressState {
                    gesture: PressGesture::Idle,
                    highlight: Highlight::Unset,
                },
                || PressMachine {
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
        self.machine.send(PressEvent::Cancel);
    }
}
