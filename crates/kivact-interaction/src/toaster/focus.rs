//! Role: Remember the external focus target and the focus held inside a toast.
use kivact_core::custom::NodeToken;
use kivact_core::state::{Machine as MachineTrait, Step};
#[derive(Clone, Default, PartialEq)]
enum ToastFocus {
    #[default]
    None,
    Inside(NodeToken),
}
#[derive(Clone, Default, PartialEq)]
pub(super) struct State {
    focus: ToastFocus,
    history_cache: FocusHistoryCache,
}
/// Last external focus observed before entry, retained as a restoration cache.
#[derive(Clone, Default, PartialEq)]
struct FocusHistoryCache(Option<NodeToken>);
impl State {
    pub(super) fn return_to(&self) -> Option<NodeToken> {
        match self.focus {
            ToastFocus::None => None,
            ToastFocus::Inside(target) => Some(target),
        }
    }
}
pub(super) enum Event {
    Observe(bool, Option<NodeToken>),
}
pub(super) enum Effect {}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let Event::Observe(inside, focused) = event;
        let previous = if !inside {
            focused.or(state.history_cache.0)
        } else {
            state.history_cache.0
        };
        let focus = if inside {
            previous.map_or(ToastFocus::None, ToastFocus::Inside)
        } else {
            ToastFocus::None
        };
        Step::to(State {
            focus,
            history_cache: FocusHistoryCache(previous),
        })
    }
}
