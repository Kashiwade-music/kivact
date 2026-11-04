//! Role: Cancel shared scalar edits when a thumb blur leaves no thumb focused.
use kivact_core::state::{Machine as MachineTrait, Step};
pub(super) struct Machine {
    pub(super) any_focused: bool,
}
pub(super) enum Event {
    ThumbFocus,
    ThumbBlur,
}
pub(super) enum Effect {
    Cancel,
}
impl MachineTrait for Machine {
    type State = ();
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, _: &(), event: Event) -> Step<(), Event, Effect> {
        match event {
            Event::ThumbBlur if !self.any_focused => Step::stay().with(Effect::Cancel),
            _ => Step::stay(),
        }
    }
}
