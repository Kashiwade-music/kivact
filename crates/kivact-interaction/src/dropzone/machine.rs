//! Role: Own drag validation and native picker lifecycle as one state.
use kivact_core::UiError;
use kivact_core::state::{Machine as MachineTrait, Step};
/// The result of validating the current drag payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Validity {
    Valid,
    Invalid(String),
}
/// The operation currently owned by a drop target.
#[derive(Clone, Debug, PartialEq)]
pub enum DropzoneState {
    Idle,
    DragOver { valid: Validity },
    Picking,
    Failed(UiError),
}
pub(super) enum Event {
    Hover(Validity),
    Leave,
    Pick,
    Complete(Result<(), UiError>),
    Accept(Vec<kivact_types::platform::FileInfo>),
}
pub(super) enum Effect {
    Files(Vec<kivact_types::platform::FileInfo>),
}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = DropzoneState;
    type Event = Event;
    type Effect = Effect;
    fn transition(
        &self,
        state: &DropzoneState,
        event: Event,
    ) -> Step<DropzoneState, Event, Effect> {
        match event {
            Event::Accept(files) => Step::to(DropzoneState::Idle).with(Effect::Files(files)),
            Event::Hover(valid) if !matches!(state, DropzoneState::Picking) => {
                Step::to(DropzoneState::DragOver { valid })
            }
            Event::Leave if matches!(state, DropzoneState::DragOver { .. }) => {
                Step::to(DropzoneState::Idle)
            }
            Event::Pick if !matches!(state, DropzoneState::Picking) => {
                Step::to(DropzoneState::Picking)
            }
            Event::Complete(Ok(())) => Step::to(DropzoneState::Idle),
            Event::Complete(Err(error)) => Step::to(DropzoneState::Failed(error)),
            _ => Step::stay(),
        }
    }
}
