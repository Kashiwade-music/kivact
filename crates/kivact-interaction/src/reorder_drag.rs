//! Role: Drag one item of a vertical list to move it one place per step.
use crate::{DragApi, DragCoordinates, DragPhase, DragState, DragUpdate, Reorder, use_drag};
use kivact_core::custom::Part;
use kivact_core::state::{Machine as MachineTrait, Step, callback, machine};
use kivact_core::view::HandlerResult;
use std::marker::PhantomData;

/// Where the dragged item is among its siblings, read at each drag update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReorderPlace {
    pub at: Option<usize>,
    pub len: usize,
}

/// Where a dragged item is: idle, or dragged with the offset at which it last moved one place.
#[derive(Clone, Copy, PartialEq)]
enum RowDrag {
    Idle,
    Dragging { applied: f64 },
}
enum RowDragEvent {
    Start,
    Update(f64),
    End,
    Cancel,
}
/// The dragged item's place, read for each event, and the step that moves it one place.
struct Machine {
    place: ReorderPlace,
    step: f64,
}
impl MachineTrait for Machine {
    type State = RowDrag;
    type Event = RowDragEvent;
    type Effect = usize;
    fn transition(
        &self,
        state: &RowDrag,
        event: RowDragEvent,
    ) -> Step<RowDrag, RowDragEvent, usize> {
        match (*state, event) {
            (_, RowDragEvent::Start) => Step::to(RowDrag::Dragging { applied: 0.0 }),
            (_, RowDragEvent::End | RowDragEvent::Cancel) => Step::to(RowDrag::Idle),
            (RowDrag::Dragging { applied }, RowDragEvent::Update(offset)) => {
                let dy = offset - applied;
                match self.place.at {
                    Some(at) if dy.abs() > self.step => {
                        let to = if dy > 0.0 {
                            (at + 1).min(self.place.len.saturating_sub(1))
                        } else {
                            at.saturating_sub(1)
                        };
                        Step::to(RowDrag::Dragging { applied: offset }).with(to)
                    }
                    _ => Step::stay(),
                }
            }
            (RowDrag::Idle, RowDragEvent::Update(_)) => Step::stay(),
        }
    }
}

/// The drag of one item of a reorderable list.
pub struct ReorderDragApi<K: 'static> {
    drag: DragApi<()>,
    key: PhantomData<fn() -> K>,
}
impl<K: 'static> Clone for ReorderDragApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: 'static> Copy for ReorderDragApi<K> {}

/// Report `Reorder { key, to }` to `on_move` each time the drag passes
/// `step` px past the last move. `place` is read at each update. Setup only.
#[track_caller]
pub fn use_reorder_drag<K: Clone + 'static, R: HandlerResult>(
    key: K,
    step: f64,
    place: impl Fn() -> ReorderPlace + 'static,
    on_move: impl Fn(Reorder<K>) -> R + 'static,
) -> ReorderDragApi<K> {
    let on_move = callback(on_move);
    let session = machine(
        RowDrag::Idle,
        move || Machine {
            place: place(),
            step,
        },
        move |_, to: usize| {
            on_move.call(Reorder {
                key: key.clone(),
                to,
            })
        },
    );
    let drag = use_drag::<()>(DragCoordinates::Window)
        .on_start(move |_| session.send(RowDragEvent::Start))
        .on_update(move |update: DragUpdate<()>| session.send(RowDragEvent::Update(update.delta.y)))
        .on_end(move |update: DragUpdate<()>| {
            session.send(if update.phase == DragPhase::Cancel {
                RowDragEvent::Cancel
            } else {
                RowDragEvent::End
            })
        });
    ReorderDragApi {
        drag,
        key: PhantomData,
    }
}

impl<K: 'static> ReorderDragApi<K> {
    /// The item's drag part. Setup only.
    #[track_caller]
    pub fn source(self) -> Part<DragState> {
        self.drag.source(())
    }
}
