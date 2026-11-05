//! Role: Menu session operations coordinate opening, focus and closing.

use super::machine::{Effect, Event, Machine, State};
use super::order::kept_levels;
use super::press_state::Press;
use super::state::{Edge, MenuEntry, MenuOptions, MenuOrigin, Opened, Target, Want};
use crate::list::typeahead::Typeahead;
use crate::{FocusReturn, PopupApi, PopupCloseReason, ReasonSet, use_popup};
use kivact_core::state::{Callback, MachineHandle, Signal, Value, memo, signal};
use kivact_core::view::NodeRef;

/// State shared by every level of one menu. All handles are `Copy`.
pub(super) struct Session<K: kivact_core::state::ItemKey> {
    pub(super) entries: Value<Vec<MenuEntry<K>>>,
    pub(super) options: crate::options_cell::OptionsCell<MenuOptions<K>>,
    /// The top level's popup; its anchor is the trigger.
    pub(super) popup: PopupApi,
    pub(super) anchor: NodeRef,
    // xtask-allow(state-fields): Read-only projection of the canonical session machine.
    /// How the open menu was opened; `None` while it is closed.
    pub(super) opened: Value<Option<Opened>>,
    // xtask-allow(state-fields): Read-only placement projection of canonical opened_by.
    /// Where the open menu was asked to appear, derived from `opened`.
    pub(super) origin: Value<Option<MenuOrigin>>,
    /// Keys of the submenu items that are open, outermost first.
    pub(super) path: Value<Vec<K>>,
    // xtask-allow(state-fields): Read-only projection of canonical per-level highlights.
    /// The item that has the focus: its depth and key.
    pub(super) focus_key: Value<Option<(usize, K)>>,
    /// How many levels the current entries keep (see `super::order::kept_levels`).
    pub(super) kept: Value<usize>,
    // xtask-allow(state-fields): Read-only projection of the canonical pending request.
    pub(super) want: Value<Option<Want<K>>>,
    pub(super) machine: MachineHandle<super::machine::Machine<K>>,
    pub(super) typed: kivact_core::state::Binding<Typeahead>,
    // xtask-allow(state-fields): Independent structural theme opt-in; created once during setup.
    pub(super) press: Option<super::press_state::Press>,
    pub(super) surface_rects: Signal<Vec<(Vec<K>, kivact_core::geometry::Rect)>>,
}

impl<K: kivact_core::state::ItemKey> Clone for Session<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for Session<K> {}

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
            State::Closed,
            move || Machine {
                entries: entries.get_untracked(),
                timing: kivact_core::style::use_theme()
                    .get()
                    .ext::<crate::InteractionPolicy>()
                    .menu
                    .submenu,
            },
            move |_, command| match command {
                Effect::Close => popup.close(PopupCloseReason::Escape),
                Effect::Closed => {
                    if let Some(press) = press {
                        press.cancel_open();
                    }
                }
            },
        );
        let state = machine.state();
        let typed = typeahead_binding(machine);
        let path = memo(move || state.with(State::path));
        let focus_key = memo(move || state.with(State::focus));
        let want = memo(move || {
            state.with(|state| match state {
                State::Closed => None,
                State::Open { want, .. } => want.clone(),
            })
        });
        let opened = memo(move || {
            state.with(|state| match state {
                State::Closed => None,
                State::Open { opened_by, .. } => Some(*opened_by),
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
            options: crate::options_cell::OptionsCell::new(MenuOptions {
                choose,
                menubar: None,
            }),
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
        machine.feed(open, Event::Popup);
        machine.feed(entries, |_| Event::Entries);
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
        self.machine.send(Event::Open(Opened::Button));
        self.popup.open_binding().set(true);
    }

    /// Open as a context menu; the focus later returns to the element that
    /// has it now.
    pub(super) fn open_at(&self, origin: MenuOrigin) {
        self.machine.send(Event::Open(Opened::Context(origin)));
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
        self.machine.send(Event::Request(Some(Want::Command {
            depth,
            target: Target::Edge(edge),
        })));
    }

    /// Ask for the focus on `key` of the level at `depth`; a pointer never
    /// replaces it.
    pub(super) fn request_key(&self, depth: usize, key: K) {
        self.machine.send(Event::Request(Some(Want::Command {
            depth,
            target: Target::Key(key),
        })));
    }

    /// The pointer rests on `key` of the level at `depth`: asked for unless a
    /// command is waiting.
    pub(super) fn hover(&self, depth: usize, key: K) {
        self.machine.send(Event::Hover(depth, key));
    }

    /// Open the submenu of `key` (an item of the level at `depth`) without
    /// moving focus.
    pub(super) fn open_hover(&self, depth: usize, key: K) {
        self.machine.send(Event::OpenSubmenu(depth, key));
    }

    /// Open the submenu of `key` and focus its first focusable item.
    pub(super) fn open_submenu(&self, depth: usize, key: K) {
        self.open_hover(depth, key);
        self.request(depth + 1, Edge::First);
    }

    /// Close the level at `depth` (not the top level) and focus the item that
    /// opened it.
    pub(super) fn close_level(&self, depth: usize) {
        let parent = self.path.with(|path| path.get(depth - 1).cloned());
        self.machine.send(Event::Truncate(depth - 1));
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

fn typeahead_binding<K: kivact_core::state::ItemKey>(
    machine: MachineHandle<Machine<K>>,
) -> kivact_core::state::Binding<Typeahead> {
    let state = machine.state();
    kivact_core::state::binding(
        move || {
            state.with(|state| match state {
                State::Closed => Typeahead::default(),
                State::Open { typeahead, .. } => typeahead.clone(),
            })
        },
        move |typed| machine.send(Event::Typeahead(typed)),
    )
}
