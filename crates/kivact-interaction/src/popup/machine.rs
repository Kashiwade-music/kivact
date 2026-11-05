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
/// Per-event host observation, never retained as popup lifecycle state.
pub(super) struct Machine {
    // xtask-allow(state-fields): Current host focus is an immutable event input, not retained popup state.
    pub(super) focused: Option<NodeToken>,
    // xtask-allow(state-fields): Current membership in the mounted content is a derived host observation.
    pub(super) focus_inside: bool,
}
impl Machine {
    fn closing(
        &self,
        opener: Option<NodeToken>,
        reason: PopupCloseReason,
        immediate: bool,
    ) -> Step<State, Event, Effect> {
        if immediate && let Some(opener) = opener {
            return Step::to(State::ReturningFocus { opener }).with(Effect::Return(opener));
        }
        Step::to(State::Closing { reason, opener })
    }
    fn may_restore(&self, target: Option<NodeToken>) -> bool {
        self.focused.is_none() || self.focus_inside || target == self.focused
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
            // A pointer can activate a non-tabstop choice while leaving focus
            // absent. An accepted programmatic close must then restore its
            // snapshotted invoker without waiting for an inside-focus cleanup.
            // A concrete outside destination keeps the ordinary close policy.
            (State::Open { .. }, Event::Accepted { reason, target }) => self.closing(
                target,
                reason,
                matches!(
                    reason,
                    PopupCloseReason::Escape
                        | PopupCloseReason::Selection
                        | PopupCloseReason::Accessibility
                ) || (reason == PopupCloseReason::Programmatic && self.may_restore(target)),
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
                let next = focus_range(*focus, anchor_present, entered);
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
            (State::Open { .. }, Event::Unmount { was_inside, target })
                if was_inside || (target.is_some() && self.may_restore(target)) =>
            {
                self.closing(target, PopupCloseReason::Unmount, true)
                    .with(Effect::Unmounted)
            }
            (
                State::Closing {
                    opener: Some(opener),
                    ..
                },
                Event::Unmount { was_inside, .. },
            ) if was_inside || self.may_restore(Some(*opener)) => {
                Step::to(State::ReturningFocus { opener: *opener })
                    .with(Effect::Return(*opener))
                    .with(Effect::Unmounted)
            }
            (_, Event::Unmount { .. }) => Step::to(State::Closed),
            _ => Step::stay(),
        }
    }
}

fn focus_range(focus: FocusRange, anchor_present: bool, entered: bool) -> FocusRange {
    let seen = matches!(
        focus,
        FocusRange::AnchorSeen
            | FocusRange::Entered {
                anchor: AnchorHistory::Seen
            }
    );
    if seen && !anchor_present {
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
        focus
    }
}
