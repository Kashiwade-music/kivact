//! Role: Pure held menu pointer transitions and owner-scoped hold timing.
use super::super::press_state::Source;
use kivact_core::geometry::Point;
use kivact_core::state::{Callback, Step};
use kivact_core::view::{NodeRef, PointerId, PointerKind};

#[derive(Clone, Copy, PartialEq)]
pub(in crate::menu) enum PressGesture {
    Idle,
    Pending {
        from: Point,
        source: PointerId,
        kind: PointerKind,
        origin: Source,
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
pub(in crate::menu) struct Input {
    pub(in crate::menu) id: PointerId,
    pub(in crate::menu) kind: PointerKind,
    pub(in crate::menu) source: Source,
}
#[derive(Clone, Copy, PartialEq)]
pub(in crate::menu) enum Highlight {
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
pub(in crate::menu) struct State {
    pub(in crate::menu) gesture: PressGesture,
    pub(in crate::menu) highlight: Highlight,
}
pub(in crate::menu) enum Event {
    Pending(Input, Point, Callback<()>),
    Hold(Input, Point, Callback<()>),
    Open(Input, Point, u64, bool),
    Move(Point, Highlight),
    Highlight(Highlight),
    Cancel,
}
pub(in crate::menu) struct Machine {
    pub(in crate::menu) now: u64,
}
type Effect = Callback<()>;
impl kivact_core::state::Machine for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Callback<()>> {
        let mut next = *state;
        match event {
            Event::Pending(input, from, callback) => {
                next.gesture = PressGesture::Pending {
                    from,
                    source: input.id,
                    kind: input.kind,
                    origin: input.source,
                };
                return Step::to(next).after(
                    std::time::Duration::from_millis(500),
                    Event::Hold(input, from, callback),
                );
            }
            Event::Hold(input, at, callback) => {
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
            Event::Open(input, at, since, entered) => {
                next.gesture = if entered {
                    PressGesture::Dragging { at, since, input }
                } else {
                    PressGesture::Holding { at, since, input }
                };
            }
            Event::Move(at, highlight) => {
                match state.gesture {
                    PressGesture::Holding { since, input, .. }
                    | PressGesture::Dragging { since, input, .. } => {
                        next.gesture = PressGesture::Dragging { at, since, input }
                    }
                    _ => return Step::stay(),
                }
                next.highlight = highlight;
            }
            Event::Highlight(highlight) => next.highlight = highlight,
            Event::Cancel => {
                next.gesture = PressGesture::Idle;
                next.highlight = Highlight::Unset;
            }
        }
        Step::to(next)
    }
}
