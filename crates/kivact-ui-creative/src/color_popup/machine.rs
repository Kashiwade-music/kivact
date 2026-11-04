//! Role: Decide edit state transitions.
use kivact::state::Step;
use kivact::style::Color;
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ColorPopupPhase {
    Closed,
    Editing { initial: Color },
}
pub(crate) struct Machine {
    pub(crate) color: Color,
}
pub(crate) enum Event {
    Open,
    Close,
    Escape,
}
pub(crate) enum Effect {
    Restore(Color),
}
impl kivact::state::Machine for Machine {
    type State = ColorPopupPhase;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &Self::State, event: Event) -> Step<Self::State, Event, Effect> {
        match (state, event) {
            (ColorPopupPhase::Closed, Event::Open) => Step::to(ColorPopupPhase::Editing {
                initial: self.color,
            }),
            (ColorPopupPhase::Editing { initial }, Event::Escape) => {
                Step::to(ColorPopupPhase::Closed).with(Effect::Restore(*initial))
            }
            (ColorPopupPhase::Editing { .. }, Event::Close) => Step::to(ColorPopupPhase::Closed),
            _ => Step::stay(),
        }
    }
}
