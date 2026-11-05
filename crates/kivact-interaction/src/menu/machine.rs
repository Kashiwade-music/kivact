//! Role: Pure menu session transitions own levels, focus requests and submenu timers.
use super::order::{item_enabled, kept_levels};
use super::state::{Edge, MenuEntry, Opened, Target, Want};
use crate::list::typeahead::Typeahead;

/// The complete state of one menu, shared by every rendered level.
#[derive(Clone, PartialEq)]
pub(super) enum State<K> {
    Closed,
    Open {
        opened_by: Opened,
        levels: Vec<SessionLevel<K>>,
        want: Option<Want<K>>,
        typeahead: Typeahead,
    },
}
#[derive(Clone, PartialEq)]
pub(super) struct SessionLevel<K> {
    // xtask-allow(state-fields): Approved SM-I08 parent identity, independent of the level highlight.
    parent: Option<K>,
    highlight: Option<K>,
    submenu: Submenu<K>,
}
#[derive(Clone, PartialEq)]
pub(super) enum Submenu<K> {
    None,
    Opening { key: K },
    Open { key: K },
    Closing { key: K },
}
impl<K: Clone> State<K> {
    pub(super) fn path(&self) -> Vec<K> {
        match self {
            Self::Closed => Vec::new(),
            Self::Open { levels, .. } => levels
                .iter()
                .skip(1)
                .filter_map(|level| level.parent.clone())
                .collect(),
        }
    }
    pub(super) fn focus(&self) -> Option<(usize, K)> {
        match self {
            Self::Closed => None,
            Self::Open { levels, .. } => levels
                .iter()
                .enumerate()
                .find_map(|(depth, level)| level.highlight.clone().map(|key| (depth, key))),
        }
    }
}
pub(super) enum Event<K> {
    Open(Opened),
    Popup(bool),
    Entries,
    Request(Option<Want<K>>),
    Hover(usize, K),
    Focus(usize, Option<K>),
    OpenSubmenu(usize, K),
    Truncate(usize),
    Typeahead(Typeahead),
    Rest(usize, K, bool, bool),
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
        let event = if matches!(event, Event::Popup(true)) && matches!(state, State::Closed) {
            Event::Open(Opened::Button)
        } else {
            event
        };
        if let Event::Open(opened_by) = event {
            match &mut next {
                State::Closed => {
                    next = State::Open {
                        opened_by,
                        levels: vec![SessionLevel {
                            parent: None,
                            highlight: None,
                            submenu: Submenu::None,
                        }],
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
        let timer = update_level_event(levels, want, typeahead, event);
        // All topology repairs happen in this transition, before publishing state.
        let path: Vec<K> = levels
            .iter()
            .skip(1)
            .filter_map(|level| level.parent.clone())
            .collect();
        let focus = levels
            .iter()
            .enumerate()
            .find_map(|(depth, level)| level.highlight.clone().map(|key| (depth, key)));
        let kept = kept_levels(&self.entries, &path, focus.as_ref());
        if kept == 0 {
            return Step::to(next).with(Effect::Close);
        }
        if kept < levels.len() {
            let parent = path
                .get(kept - 1)
                .cloned()
                .filter(|key| item_enabled(&self.entries, &path[..kept - 1], key));
            truncate(levels, kept - 1);
            for level in levels.iter_mut() {
                level.highlight = None;
            }
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
fn open_level<K: Clone + PartialEq>(levels: &mut Vec<SessionLevel<K>>, depth: usize, key: K) {
    if depth >= levels.len() {
        return;
    }
    if levels.len() == depth + 2 && levels[depth + 1].parent.as_ref() == Some(&key) {
        return;
    }
    levels.truncate(depth + 1);
    levels[depth].submenu = Submenu::Open { key: key.clone() };
    levels.push(SessionLevel {
        parent: Some(key),
        highlight: None,
        submenu: Submenu::None,
    });
}
fn truncate<K>(levels: &mut Vec<SessionLevel<K>>, depth: usize) {
    levels.truncate(depth + 1);
    if let Some(level) = levels.get_mut(depth) {
        level.submenu = Submenu::None;
    }
}

fn update_level_event<K: kivact_core::state::ItemKey>(
    levels: &mut Vec<SessionLevel<K>>,
    want: &mut Option<Want<K>>,
    typeahead: &mut Typeahead,
    event: Event<K>,
) -> Option<(bool, usize, K)> {
    let mut timer = None;
    match event {
            Event::Request(request) => *want = request,
            Event::Hover(depth, key) if !matches!(want, Some(Want::Command { .. })) => *want = Some(Want::Hover { depth, key }),
            Event::Focus(depth, key) => {
                for level in levels.iter_mut() { level.highlight = None; }
                if let Some(level) = levels.get_mut(depth) { level.highlight = key; }
            }
            Event::OpenSubmenu(depth, key) => open_level(levels, depth, key),
            Event::Truncate(depth) => truncate(levels, depth),
            Event::Typeahead(typed) => *typeahead = typed,
            Event::Rest(depth, key, over, opens) => {
                let parent = levels.get(depth + 1).and_then(|level| level.parent.clone());
                if let Some(level) = levels.get_mut(depth) {
                    if opens {
                        if over && !matches!(&level.submenu, Submenu::Open { key: current } if current == &key) {
                            level.submenu = Submenu::Opening { key: key.clone() };
                            timer = Some((true, depth, key));
                        } else if !over && matches!(&level.submenu, Submenu::Opening { key: current } if current == &key) {
                            level.submenu = Submenu::None;
                        }
                    } else if over && let Some(parent) = parent {
                        level.submenu = Submenu::Closing { key: parent.clone() };
                        timer = Some((false, depth, parent));
                    } else if !over && let Submenu::Closing { key } = &level.submenu {
                        level.submenu = Submenu::Open { key: key.clone() };
                    }
                }
            }
            Event::OpenAfter(depth, key) => {
                if levels.get(depth).is_some_and(|level| matches!(&level.submenu, Submenu::Opening { key: current } if current == &key)) { open_level(levels, depth, key); }
            }
            Event::CloseAfter(depth, key)
                if levels.get(depth).is_some_and(|level| matches!(&level.submenu, Submenu::Closing { key: current } if current == &key)) => { truncate(levels, depth); }
            _ => {}
        }
    timer
}
