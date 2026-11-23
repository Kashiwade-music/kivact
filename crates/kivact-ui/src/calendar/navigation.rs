//! Role: The calendar's one-shot initial focus.
use kivact_core::state::{Machine, Step};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum InitialFocus {
    Pending,
    Applied,
}

pub(super) struct FocusSnapshot {
    pub(super) requested: bool,
}

impl Machine for FocusSnapshot {
    type State = InitialFocus;
    type Event = kivact_interaction::CalendarGridApi;
    type Effect = kivact_interaction::CalendarGridApi;
    fn transition(
        &self,
        state: &InitialFocus,
        grid: Self::Event,
    ) -> Step<InitialFocus, Self::Event, Self::Effect> {
        match state {
            InitialFocus::Applied => Step::stay(),
            InitialFocus::Pending => {
                let next = Step::to(InitialFocus::Applied);
                if self.requested {
                    next.with(grid)
                } else {
                    next
                }
            }
        }
    }
}
