//! Role: Menu session operations coordinate opening, focus and closing.

use super::order::{item_enabled, kept_levels};
use super::press_state::Press;
use super::state::{Edge, MenuEntry, MenuOrigin, Opened, Session, Target, Want};
use crate::list::typeahead::Typeahead;
use crate::{FocusReturn, PopupCloseReason, ReasonSet, use_popup};
use kivact_core::state::{Callback, Value, memo, signal};

impl<K: kivact_core::state::ItemKey> Session<K> {
    /// Setup only.
    #[track_caller]
    pub(super) fn new(entries: Value<Vec<MenuEntry<K>>>, choose: Callback<K>) -> Self {
        let open = signal(false);
        // The popup returns the focus; each opening says where to (see
        // `Session::focus_return`, installed when the top level binds).
        let popup = use_popup(open);
        let anchor = popup.anchor_ref();
        let press = (kivact_core::style::use_theme()
            .get()
            .ext::<crate::InteractionPolicy>()
            .menu
            .press)
            .0
            .then(Press::new);
        let machine = kivact_core::state::machine(
            MenuSession::Closed,
            move || SessionMachine {
                entries: entries.get_untracked(),
                timing: kivact_core::style::use_theme()
                    .get()
                    .ext::<crate::InteractionPolicy>()
                    .menu
                    .submenu,
            },
            move |_, command| match command {
                SessionEffect::Close => popup.close(PopupCloseReason::Escape),
                SessionEffect::Closed => {
                    if let Some(press) = press {
                        press.cancel_open();
                    }
                }
            },
        );
        let state = machine.state();
        let typed = kivact_core::state::binding(
            move || {
                state.with(|state| match state {
                    MenuSession::Closed => Typeahead::default(),
                    MenuSession::Open { typeahead, .. } => typeahead.clone(),
                })
            },
            move |typed| machine.send(SessionEvent::Typeahead(typed)),
        );
        let path = memo(move || state.with(MenuSession::path));
        let focus_key = memo(move || state.with(MenuSession::focus));
        let want = memo(move || {
            state.with(|state| match state {
                MenuSession::Closed => None,
                MenuSession::Open { want, .. } => want.clone(),
            })
        });
        let opened = memo(move || {
            state.with(|state| match state {
                MenuSession::Closed => None,
                MenuSession::Open { opened_by, .. } => Some(*opened_by),
            })
        });
        let origin = memo(move || {
            opened.get().map(|opened| match opened {
                Opened::Button => MenuOrigin::Anchor,
                Opened::Context(origin) => origin,
            })
        });
        let kept = memo(move || {
            entries.with(|entries| {
                state.with(|state| kept_levels(entries, &state.path(), state.focus().as_ref()))
            })
        });
        let this = Self {
            entries,
            choose: kivact_core::state::var(choose),
            popup,
            anchor,
            opened,
            origin,
            path,
            focus_key,
            kept,
            want,
            typed,
            machine,
            surface_rects: signal(Vec::new()),
            press,
        };
        machine.feed(open, SessionEvent::Popup);
        machine.feed(entries, |_| SessionEvent::Entries);
        this
    }

    pub(super) fn is_open(&self) -> Value<bool> {
        self.popup.is_open()
    }

    /// Close the whole menu as `reason`. After Escape, Selection and
    /// Accessibility the focus goes back to the element that had it before a
    /// context menu opened, or else to the trigger.
    pub(super) fn close(&self, reason: PopupCloseReason) {
        if !self.is_open().get_untracked() {
            return;
        }
        if let Some(press) = self.press {
            press.cancel();
        }
        self.popup.close(reason);
    }

    /// Where the focus returns for the opening in progress: to the trigger
    /// for a menu button, to the element that had the focus for a context
    /// menu.
    pub(super) fn focus_return(&self) -> FocusReturn {
        match self.opened.get_untracked() {
            Some(Opened::Context(_)) => FocusReturn::opener(ReasonSet::DISMISS),
            Some(Opened::Button) | None => FocusReturn::anchor(self.anchor),
        }
    }

    /// Open from the menu button; the focus later returns to it.
    pub(super) fn open_from_button(&self) {
        self.machine.send(SessionEvent::Open(Opened::Button));
        self.popup.open_binding().set(true);
    }

    /// Open as a context menu; the focus later returns to the element that
    /// has it now.
    pub(super) fn open_at(&self, origin: MenuOrigin) {
        self.machine
            .send(SessionEvent::Open(Opened::Context(origin)));
        self.popup.open_binding().set(true);
    }
}

impl<K: kivact_core::state::ItemKey> Session<K> {
    /// The key of the focused item if the focus is on the level at `depth`.
    pub(super) fn focused_at(&self, depth: usize) -> Option<K> {
        self.focus_key.with(|focus| match focus {
            Some((at, key)) if *at == depth => Some(key.clone()),
            _ => None,
        })
    }

    pub(super) fn request(&self, depth: usize, edge: Edge) {
        self.machine.send(SessionEvent::Request(Some(Want::Command {
            depth,
            target: Target::Edge(edge),
        })));
    }

    /// Ask for the focus on `key` of the level at `depth`; a pointer never
    /// replaces it.
    pub(super) fn request_key(&self, depth: usize, key: K) {
        self.machine.send(SessionEvent::Request(Some(Want::Command {
            depth,
            target: Target::Key(key),
        })));
    }

    /// The pointer rests on `key` of the level at `depth`: asked for unless a
    /// command is waiting.
    pub(super) fn hover(&self, depth: usize, key: K) {
        self.machine.send(SessionEvent::Hover(depth, key));
    }

    /// Open the submenu of `key` (an item of the level at `depth`) without
    /// moving focus.
    pub(super) fn open_hover(&self, depth: usize, key: K) {
        self.machine.send(SessionEvent::OpenSubmenu(depth, key));
    }

    /// Open the submenu of `key` and focus its first enabled item.
    pub(super) fn open_submenu(&self, depth: usize, key: K) {
        self.open_hover(depth, key);
        self.request(depth + 1, Edge::First);
    }

    /// Close every level deeper than `depth`.
    pub(super) fn close_below(&self, depth: usize) {
        self.machine.send(SessionEvent::Truncate(depth));
    }

    /// Close the level at `depth` (not the top level) and focus the item that
    /// opened it.
    pub(super) fn close_level(&self, depth: usize) {
        let parent = self.path.with(|path| path.get(depth - 1).cloned());
        self.machine.send(SessionEvent::Truncate(depth - 1));
        if let Some(parent) = parent {
            self.request_key(depth - 1, parent);
        }
    }

    /// Escape: close the level at `depth`, or the whole menu at the top.
    pub(super) fn leave(&self, depth: usize) {
        if depth == 0 {
            self.close(PopupCloseReason::Escape);
        } else {
            self.close_level(depth);
        }
    }
}

/// The complete state of one menu, shared by every rendered level.
#[derive(Clone, PartialEq)]
pub(super) enum MenuSession<K> {
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
impl<K: Clone> MenuSession<K> {
    fn path(&self) -> Vec<K> {
        match self {
            Self::Closed => Vec::new(),
            Self::Open { levels, .. } => levels
                .iter()
                .skip(1)
                .filter_map(|level| level.parent.clone())
                .collect(),
        }
    }
    fn focus(&self) -> Option<(usize, K)> {
        match self {
            Self::Closed => None,
            Self::Open { levels, .. } => levels
                .iter()
                .enumerate()
                .find_map(|(depth, level)| level.highlight.clone().map(|key| (depth, key))),
        }
    }
}
pub(super) enum SessionEvent<K> {
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
pub(super) enum SessionEffect {
    Close,
    Closed,
}
pub(super) struct SessionMachine<K: kivact_core::state::ItemKey> {
    entries: Vec<MenuEntry<K>>,
    timing: crate::menu_policy::SubmenuTiming,
}
impl<K: kivact_core::state::ItemKey> kivact_core::state::Machine for SessionMachine<K> {
    type State = MenuSession<K>;
    type Event = SessionEvent<K>;
    type Effect = SessionEffect;
    fn transition(
        &self,
        state: &Self::State,
        event: Self::Event,
    ) -> kivact_core::state::Step<Self::State, Self::Event, Self::Effect> {
        use kivact_core::state::Step;
        let mut next = state.clone();
        if let SessionEvent::Popup(false) = event {
            return Step::to(MenuSession::Closed).with(SessionEffect::Closed);
        }
        if let SessionEvent::Open(opened_by) = event {
            match &mut next {
                MenuSession::Closed => {
                    next = MenuSession::Open {
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
                MenuSession::Open {
                    opened_by: opened, ..
                } => *opened = opened_by,
            }
        }
        let MenuSession::Open {
            levels,
            want,
            typeahead,
            ..
        } = &mut next
        else {
            return Step::stay();
        };
        let mut timer = None;
        match event {
            SessionEvent::Request(request) => *want = request,
            SessionEvent::Hover(depth, key) if !matches!(want, Some(Want::Command { .. })) => *want = Some(Want::Hover { depth, key }),
            SessionEvent::Focus(depth, key) => {
                for level in levels.iter_mut() { level.highlight = None; }
                if let Some(level) = levels.get_mut(depth) { level.highlight = key; }
            }
            SessionEvent::OpenSubmenu(depth, key) => open_level(levels, depth, key),
            SessionEvent::Truncate(depth) => truncate(levels, depth),
            SessionEvent::Typeahead(typed) => *typeahead = typed,
            SessionEvent::Rest(depth, key, over, opens) => {
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
            SessionEvent::OpenAfter(depth, key) => {
                if levels.get(depth).is_some_and(|level| matches!(&level.submenu, Submenu::Opening { key: current } if current == &key)) { open_level(levels, depth, key); }
            }
            SessionEvent::CloseAfter(depth, key) => {
                if levels.get(depth).is_some_and(|level| matches!(&level.submenu, Submenu::Closing { key: current } if current == &key)) { truncate(levels, depth); }
            }
            _ => {}
        }
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
            return Step::to(MenuSession::Closed).with(SessionEffect::Close);
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
                step.after(timing.open, SessionEvent::OpenAfter(depth, key))
            } else {
                step.after(timing.close, SessionEvent::CloseAfter(depth, key))
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
