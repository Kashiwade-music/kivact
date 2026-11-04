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
    Close(PopupCloseReason),
    Accepted(PopupCloseReason),
    Focus {
        anchor_present: bool,
        entered: bool,
        outside: bool,
    },
    Unmount {
        was_inside: bool,
    },
    Returned,
}
pub(super) enum Effect {
    Misuse,
    WriteClose(PopupCloseReason),
    Return(NodeToken),
    Unmounted,
}
pub(super) struct Machine {
    pub modal: bool,
    pub anchor: Option<NodeToken>,
}
impl Machine {
    fn closing(
        &self,
        opener: Option<NodeToken>,
        policy: FocusPolicy,
        reason: PopupCloseReason,
        immediate: bool,
    ) -> Step<State, Event, Effect> {
        let target = if !self.modal && policy.0.restores(reason) {
            match policy.0.target {
                FocusReturnTarget::Opener => opener.or(self.anchor),
                FocusReturnTarget::Anchor(reference) => {
                    use kivact_core::custom::NodeRefExt;
                    reference.mounted_node()
                }
            }
        } else {
            None
        };
        if immediate {
            if let Some(opener) = target {
                return Step::to(State::ReturningFocus { opener }).with(Effect::Return(opener));
            }
        }
        Step::to(State::Closing {
            reason,
            opener: target,
        })
    }
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match (state, event) {
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
            (_, Event::Close(reason)) => Step::stay().with(Effect::WriteClose(reason)),
            (State::Open { opener, policy, .. }, Event::Accepted(reason)) => self.closing(
                *opener,
                *policy,
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
                    step.with(Effect::WriteClose(PopupCloseReason::AnchorRemoved))
                } else if matches!(next, FocusRange::Entered { .. }) && outside && !entered {
                    step.with(Effect::WriteClose(PopupCloseReason::Tab))
                } else {
                    step
                }
            }
            (State::Open { opener, policy, .. }, Event::Unmount { was_inside: true }) => {
                let step = self.closing(*opener, *policy, PopupCloseReason::Unmount, true);
                step.with(Effect::Unmounted)
            }
            (
                State::Closing {
                    opener: Some(opener),
                    ..
                },
                Event::Unmount { was_inside: true },
            ) => Step::to(State::ReturningFocus { opener: *opener })
                .with(Effect::Return(*opener))
                .with(Effect::Unmounted),
            (_, Event::Unmount { .. }) => Step::to(State::Closed),
            (State::ReturningFocus { .. }, Event::Returned) => Step::stay(),
            _ => Step::stay(),
        }
    }
}
