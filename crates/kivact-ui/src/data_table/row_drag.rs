//! Role: Decide where a dragged data-table row is and when it moves one place.
use kivact_core::state::{Machine, Step};

/// Where a dragged row is: idle, or dragged with the offset at which it last moved one place.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum RowDrag {
    Idle,
    Dragging { applied: f64 },
}
pub(super) enum RowDragEvent {
    Start,
    Update(f64),
    End,
    Cancel,
}
/// The dragged row's place among the caller's rows, read for each event.
pub(super) struct RowPlace {
    pub(super) at: Option<usize>,
    pub(super) len: usize,
}
impl Machine for RowPlace {
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
                match self.at {
                    Some(at) if dy.abs() > 28.0 => {
                        let to = if dy > 0.0 {
                            (at + 1).min(self.len.saturating_sub(1))
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
