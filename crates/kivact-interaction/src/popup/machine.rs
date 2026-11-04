//! Role: Compute one popup surface's focus range, dismissal and restoration.
use super::super::focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason};
use kivact_core::custom::NodeToken;
use kivact_core::state::{Machine as MachineTrait, Step};

#[derive(Clone, Copy)]
pub(super) struct FocusPolicy(pub FocusReturn);
impl PartialEq for FocusPolicy {
    fn eq(&self, other: &Self) -> bool {
        self.0.on == other.0.on
            && match (self.0.target, other.0.target) {
                (FocusReturnTarget::Opener, FocusReturnTarget::Opener) => true,
                (FocusReturnTarget::Anchor(a), FocusReturnTarget::Anchor(b)) => a == b,
                _ => false,
            }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AnchorHistory {
    Unseen,
    Seen,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FocusRange {
    NotEntered,
    AnchorSeen,
    Entered { anchor: AnchorHistory },
    AnchorRemoved,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum State {
    Closed,
    Open {
        opener: Option<NodeToken>,
        policy: FocusPolicy,
        focus: FocusRange,
    },
    Closing {
        reason: PopupCloseReason,
        opener: Option<NodeToken>,
    },
    ReturningFocus {
        opener: NodeToken,
    },
}
pub(super) enum Event {
    Open {
        opener: Option<NodeToken>,
        policy: FocusReturn,
    },
    Policy(FocusReturn),
    FocusInitial,
    Close(PopupCloseReason),
    Accepted {
        reason: PopupCloseReason,
        target: Option<NodeToken>,
    },
    Focus {
        anchor_present: bool,
        entered: bool,
        outside: bool,
    },
    Unmount {
        was_inside: bool,
        target: Option<NodeToken>,
    },
}
pub(super) enum Effect {
    Misuse,
    FocusInitial,
    WriteClose {
        reason: PopupCloseReason,
        opening: Option<(Option<NodeToken>, FocusPolicy)>,
    },
    Return(NodeToken),
    Unmounted,
}
pub(super) struct Machine;
impl Machine {
    fn closing(
        &self,
        opener: Option<NodeToken>,
        reason: PopupCloseReason,
        immediate: bool,
    ) -> Step<State, Event, Effect> {
        if immediate {
            if let Some(opener) = opener {
                return Step::to(State::ReturningFocus { opener }).with(Effect::Return(opener));
            }
        }
        Step::to(State::Closing { reason, opener })
    }
    fn close_effect(&self, state: &State, reason: PopupCloseReason) -> Effect {
        let opening = match state {
            State::Open { opener, policy, .. } => Some((*opener, *policy)),
            _ => None,
        };
        Effect::WriteClose { reason, opening }
    }
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match (state, event) {
            (_, Event::FocusInitial) => Step::stay().with(Effect::FocusInitial),
            (State::Open { .. }, Event::Open { .. }) => Step::stay().with(Effect::Misuse),
            (_, Event::Open { opener, policy }) => Step::to(State::Open {
                opener,
                policy: FocusPolicy(policy),
                focus: FocusRange::NotEntered,
            }),
            (State::Open { opener, focus, .. }, Event::Policy(policy)) => Step::to(State::Open {
                opener: *opener,
                policy: FocusPolicy(policy),
                focus: *focus,
            }),
            (_, Event::Close(reason)) => Step::stay().with(self.close_effect(state, reason)),
            (State::Open { .. }, Event::Accepted { reason, target }) => self.closing(
                target,
                reason,
                matches!(
                    reason,
                    PopupCloseReason::Escape
                        | PopupCloseReason::Selection
                        | PopupCloseReason::Accessibility
                ),
            ),
            (
                State::Open {
                    opener,
                    policy,
                    focus,
                },
                Event::Focus {
                    anchor_present,
                    entered,
                    outside,
                },
            ) => {
                let seen = matches!(
                    focus,
                    FocusRange::AnchorSeen
                        | FocusRange::Entered {
                            anchor: AnchorHistory::Seen
                        }
                );
                let next = if seen && !anchor_present {
                    FocusRange::AnchorRemoved
                } else if entered || matches!(focus, FocusRange::Entered { .. }) {
                    FocusRange::Entered {
                        anchor: if seen || anchor_present {
                            AnchorHistory::Seen
                        } else {
                            AnchorHistory::Unseen
                        },
                    }
                } else if anchor_present {
                    FocusRange::AnchorSeen
                } else {
                    *focus
                };
                let step = Step::to(State::Open {
                    opener: *opener,
                    policy: *policy,
                    focus: next,
                });
                if next == FocusRange::AnchorRemoved {
                    step.with(self.close_effect(state, PopupCloseReason::AnchorRemoved))
                } else if matches!(next, FocusRange::Entered { .. }) && outside && !entered {
                    step.with(self.close_effect(state, PopupCloseReason::Tab))
                } else {
                    step
                }
            }
            (
                State::Open { .. },
                Event::Unmount {
                    was_inside: true,
                    target,
                },
            ) => self
                .closing(target, PopupCloseReason::Unmount, true)
                .with(Effect::Unmounted),
            (
                State::Closing {
                    opener: Some(opener),
                    ..
                },
                Event::Unmount {
                    was_inside: true, ..
                },
            ) => Step::to(State::ReturningFocus { opener: *opener })
                .with(Effect::Return(*opener))
                .with(Effect::Unmounted),
            (_, Event::Unmount { .. }) => Step::to(State::Closed),
            _ => Step::stay(),
        }
    }
}
