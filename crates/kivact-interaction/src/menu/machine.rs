//! Role: Pure menu session transitions own levels, focus requests and submenu timers.
use super::order::{item_enabled, kept_levels};
use super::state::{Edge, MenuEntry, Opened, PressOver, Target, Want};
use crate::list::typeahead::Typeahead;

/// The complete state of one menu, shared by every rendered level.
#[derive(Clone, PartialEq)]
pub(super) enum State<K> {
    Closed,
    Open {
        opened_by: Opened,
        levels: Levels<K>,
        want: Option<Want<K>>,
        typeahead: Typeahead,
    },
}
/// The open levels of one menu and its one highlighted item. Level `d + 1`
/// is the submenu of the item `submenus[d].key` of level `d`; every level
/// has its own submenu timer.
#[derive(Clone, PartialEq)]
pub(super) struct Levels<K> {
    /// The submenu timer of the top level.
    top: SubmenuTimer<K>,
    /// The open submenus, outermost first.
    submenus: Vec<OpenSubmenu<K>>,
    /// The highlighted item of all levels: its depth (an open level) and key.
    focus: Option<(usize, K)>,
}
/// An open submenu: the key of the item that opened it and the submenu
/// timer of its own level.
#[derive(Clone, PartialEq)]
struct OpenSubmenu<K> {
    key: K,
    timer: SubmenuTimer<K>,
}
/// The pending submenu timer of one level. The level's open submenu is
/// `Levels::child`; the timer does not store it again.
#[derive(Clone, PartialEq)]
enum SubmenuTimer<K> {
    Idle,
    /// The submenu of `key` opens after the delay because the pointer rests on it.
    Opening {
        key: K,
    },
    /// The open submenu closes after the delay because the pointer rests
    /// on the plain item `by`; only leaving `by` keeps it open.
    Closing {
        by: K,
    },
}
impl<K: Clone> State<K> {
    pub(super) fn path(&self) -> Vec<K> {
        match self {
            Self::Closed => Vec::new(),
            Self::Open { levels, .. } => levels.path(),
        }
    }
    pub(super) fn focus(&self) -> Option<(usize, K)> {
        match self {
            Self::Closed => None,
            Self::Open { levels, .. } => levels.focus.clone(),
        }
    }
}
impl<K: Clone> Levels<K> {
    fn new() -> Self {
        Self {
            top: SubmenuTimer::Idle,
            submenus: Vec::new(),
            focus: None,
        }
    }
    /// How many levels are open, the top level included.
    fn len(&self) -> usize {
        self.submenus.len() + 1
    }
    /// The keys of the submenu items that are open, outermost first.
    fn path(&self) -> Vec<K> {
        self.submenus
            .iter()
            .map(|submenu| submenu.key.clone())
            .collect()
    }
    /// The key of the item of level `depth` whose submenu is open.
    fn child(&self, depth: usize) -> Option<&K> {
        self.submenus.get(depth).map(|submenu| &submenu.key)
    }
    fn timer(&self, depth: usize) -> Option<&SubmenuTimer<K>> {
        match depth {
            0 => Some(&self.top),
            _ => self.submenus.get(depth - 1).map(|submenu| &submenu.timer),
        }
    }
    fn timer_mut(&mut self, depth: usize) -> Option<&mut SubmenuTimer<K>> {
        match depth {
            0 => Some(&mut self.top),
            _ => self
                .submenus
                .get_mut(depth - 1)
                .map(|submenu| &mut submenu.timer),
        }
    }
    /// Highlight `key` of the level at `depth`, or nothing.
    fn set_focus(&mut self, depth: usize, key: Option<K>) {
        self.focus = if depth < self.len() {
            key.map(|key| (depth, key))
        } else {
            None
        };
    }
    /// Keep the levels up to `depth` and stop the submenu timer of level `depth`.
    fn truncate(&mut self, depth: usize) {
        self.submenus.truncate(depth);
        if self.focus.as_ref().is_some_and(|(at, _)| *at > depth) {
            self.focus = None;
        }
        if let Some(timer) = self.timer_mut(depth) {
            *timer = SubmenuTimer::Idle;
        }
    }
}
impl<K: Clone + PartialEq> Levels<K> {
    /// Open the submenu of `key`, an item of level `depth`, as the deepest level.
    fn open(&mut self, depth: usize, key: K) {
        if depth >= self.len() {
            return;
        }
        if self.len() == depth + 2 && self.child(depth) == Some(&key) {
            return;
        }
        self.truncate(depth);
        self.submenus.push(OpenSubmenu {
            key,
            timer: SubmenuTimer::Idle,
        });
    }
    /// The pointer rests on (`over`) or leaves the item `key` of level
    /// `depth`; `opens` when it is a submenu item. Returns the timer to
    /// start: whether it opens, the depth and the key it names.
    fn rest(&mut self, depth: usize, key: K, over: bool, opens: bool) -> Option<(bool, usize, K)> {
        let child = self.child(depth).cloned();
        let timer = self.timer_mut(depth)?;
        if opens {
            let shown = matches!(timer, SubmenuTimer::Idle) && child.as_ref() == Some(&key);
            if over && !shown {
                *timer = SubmenuTimer::Opening { key: key.clone() };
                return Some((true, depth, key));
            }
            if !over && matches!(timer, SubmenuTimer::Opening { key: current } if *current == key) {
                *timer = SubmenuTimer::Idle;
            }
        } else if over && let Some(child) = child {
            *timer = SubmenuTimer::Closing { by: key };
            return Some((false, depth, child));
        } else if !over && matches!(timer, SubmenuTimer::Closing { by } if *by == key) {
            *timer = SubmenuTimer::Idle;
        }
        None
    }
}
pub(super) enum Event<K> {
    Open(Opened),
    Popup(bool),
    Entries,
    Request(Option<Want<K>>),
    Pointer {
        depth: usize,
        key: K,
        submenu: bool,
        hovered: bool,
        press: PressOver,
    },
    Focus(usize, Option<K>),
    OpenSubmenu(usize, K),
    Truncate(usize),
    Typeahead(Typeahead),
    /// The type-to-find window ended.
    Expire,
    OpenAfter(usize, K),
    CloseAfter(usize, K),
}
pub(super) enum Effect {
    Close,
    Closed,
}
pub(super) struct Machine<K: kivact_core::state::ItemKey> {
    pub(super) entries: Vec<MenuEntry<K>>,
    pub(super) timing: crate::menu_policy::SubmenuTiming,
}
impl<K: kivact_core::state::ItemKey> kivact_core::state::Machine for Machine<K> {
    type State = State<K>;
    type Event = Event<K>;
    type Effect = Effect;
    fn transition(
        &self,
        state: &Self::State,
        event: Self::Event,
    ) -> kivact_core::state::Step<Self::State, Self::Event, Self::Effect> {
        use kivact_core::state::Step;
        let mut next = state.clone();
        if let Event::Popup(false) = event {
            return Step::to(State::Closed).with(Effect::Closed);
        }
        let event = accepted_opening(state, event);
        if let Event::Open(opened_by) = event {
            match &mut next {
                State::Closed => {
                    next = State::Open {
                        opened_by,
                        levels: Levels::new(),
                        want: None,
                        typeahead: Typeahead::default(),
                    }
                }
                State::Open {
                    opened_by: opened, ..
                } => *opened = opened_by,
            }
        }
        let State::Open {
            levels,
            want,
            typeahead,
            ..
        } = &mut next
        else {
            return Step::stay();
        };
        let typing = matches!(&event, Event::Typeahead(Typeahead::Typing { .. }));
        let timer = update_level_event(levels, want, typeahead, event);
        // All topology repairs happen in this transition, before publishing state.
        let path = levels.path();
        let kept = kept_levels(&self.entries, &path, levels.focus.as_ref());
        if kept == 0 {
            return Step::to(next).with(Effect::Close);
        }
        if kept < levels.len() {
            let parent = path
                .get(kept - 1)
                .cloned()
                .filter(|key| item_enabled(&self.entries, &path[..kept - 1], key));
            levels.truncate(kept - 1);
            levels.focus = None;
            *want = Some(Want::Command {
                depth: kept - 1,
                target: parent.map_or(Target::Edge(Edge::First), Target::Key),
            });
        }
        if want
            .as_ref()
            .is_some_and(|want| want.depth() >= levels.len())
        {
            *want = None;
        }
        let mut step = Step::to(next);
        if typing {
            step = step.after(crate::list::typeahead::TYPEAHEAD_WINDOW, Event::Expire);
        }
        if let Some((opens, depth, key)) = timer {
            let timing = self.timing;
            step = if opens {
                step.after(timing.open, Event::OpenAfter(depth, key))
            } else {
                step.after(timing.close, Event::CloseAfter(depth, key))
            };
        }
        step
    }
}

fn update_level_event<K: kivact_core::state::ItemKey>(
    levels: &mut Levels<K>,
    want: &mut Option<Want<K>>,
    typeahead: &mut Typeahead,
    event: Event<K>,
) -> Option<(bool, usize, K)> {
    match event {
        Event::Request(request) => *want = request,
        Event::Pointer {
            depth,
            key,
            submenu: opens,
            hovered,
            press,
        } => {
            let over = match press {
                PressOver::Over => true,
                PressOver::Away => false,
                PressOver::None => hovered,
            };
            let timer = levels.rest(depth, key.clone(), over, opens);
            if hovered && !matches!(want, Some(Want::Command { .. })) {
                *want = Some(Want::Hover { depth, key });
            }
            return timer;
        }
        Event::Focus(depth, key) => levels.set_focus(depth, key),
        Event::OpenSubmenu(depth, key) => levels.open(depth, key),
        Event::Truncate(depth) => levels.truncate(depth),
        Event::Typeahead(typed) => *typeahead = typed,
        Event::Expire => *typeahead = Typeahead::Idle,
        Event::OpenAfter(depth, key) => {
            if matches!(levels.timer(depth), Some(SubmenuTimer::Opening { key: current }) if *current == key)
            {
                levels.open(depth, key);
            }
        }
        Event::CloseAfter(depth, key)
            if matches!(levels.timer(depth), Some(SubmenuTimer::Closing { .. }))
                && levels.child(depth) == Some(&key) =>
        {
            levels.truncate(depth);
        }
        _ => {}
    }
    None
}

/// Interpret an accepted external open as a menu opening without a focus request.
fn accepted_opening<K>(state: &State<K>, event: Event<K>) -> Event<K> {
    if matches!(event, Event::Popup(true)) && matches!(state, State::Closed) {
        Event::Open(Opened::External)
    } else {
        event
    }
}
