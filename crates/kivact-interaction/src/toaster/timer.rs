//! Role: Own one toast's focus restoration and independently paused deadline.
use kivact_core::custom::NodeToken;
use kivact_core::state::{Machine as MachineTrait, Step};
use std::time::Duration;
#[derive(Clone, PartialEq)]
enum ToastTimer {
    Running { deadline_ns: u64 },
    Paused { left_ns: u64 },
    Expired,
}
#[derive(Clone, Default, PartialEq)]
enum ToastFocus {
    #[default]
    None,
    Inside(NodeToken),
}
/// Last external focus observed before entry, retained as a restoration cache.
#[derive(Clone, Default, PartialEq)]
struct FocusHistoryCache(Option<NodeToken>);
#[derive(Clone, PartialEq)]
pub(super) struct State {
    timer: ToastTimer,
    focus: ToastFocus,
    history_cache: FocusHistoryCache,
}
impl State {
    pub(super) fn new(left_ns: u64) -> Self {
        Self {
            timer: ToastTimer::Paused { left_ns },
            focus: ToastFocus::None,
            history_cache: FocusHistoryCache::default(),
        }
    }
    pub(super) fn return_to(&self) -> Option<NodeToken> {
        match self.focus {
            ToastFocus::None => None,
            ToastFocus::Inside(target) => Some(target),
        }
    }
}
pub(super) enum Event {
    Observe {
        inside: bool,
        focused: Option<NodeToken>,
        engaged: bool,
    },
    Expire,
}
pub(super) enum Effect {
    Dismiss,
}
pub(super) struct Machine {
    pub now_ns: u64,
    pub timed: bool,
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let mut next = state.clone();
        let Event::Observe {
            inside,
            focused,
            engaged,
        } = event
        else {
            if matches!(state.timer, ToastTimer::Running { .. }) {
                next.timer = ToastTimer::Expired;
                return Step::to(next).with(Effect::Dismiss);
            }
            return Step::stay();
        };
        if !inside {
            next.history_cache.0 = focused.or(next.history_cache.0);
        }
        next.focus = if inside {
            next.history_cache
                .0
                .map_or(ToastFocus::None, ToastFocus::Inside)
        } else {
            ToastFocus::None
        };
        if !self.timed {
            return Step::to(next);
        }
        match (&state.timer, engaged) {
            (ToastTimer::Running { deadline_ns }, true) => {
                next.timer = ToastTimer::Paused {
                    left_ns: deadline_ns.saturating_sub(self.now_ns),
                };
                Step::to(next).cancel(Event::Expire)
            }
            (ToastTimer::Paused { left_ns }, false) => {
                next.timer = ToastTimer::Running {
                    deadline_ns: self.now_ns.saturating_add(*left_ns),
                };
                Step::to(next).after(Duration::from_nanos(*left_ns), Event::Expire)
            }
            _ => Step::to(next),
        }
    }
}
