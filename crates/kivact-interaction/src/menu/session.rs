//! Role: Menu session operations coordinate opening, focus and closing.

use super::order::{item_enabled, kept_levels};
use super::press_state::{MenuPress, Press};
use super::state::{Edge, MenuEntry, MenuOrigin, Opened, Session, Target, Want};
use crate::list::typeahead::Typeahead;
use crate::{FocusReturn, PopupCloseReason, ReasonSet, use_popup};
use kivact_core::state::{Callback, Value, effect, memo, signal, var};

impl<K: kivact_core::state::ItemKey> Session<K> {
    /// Setup only.
    #[track_caller]
    pub(super) fn new(entries: Value<Vec<MenuEntry<K>>>, choose: Callback<K>) -> Self {
        let open = signal(false);
        // The popup returns the focus; each opening says where to (see
        // `Session::focus_return`, installed when the top level binds).
        let popup = use_popup(open);
        let anchor = popup.anchor_ref();
        let path = signal(Vec::<K>::new());
        let focus_key = signal(None::<(usize, K)>);
        let want = signal(None::<Want<K>>);
        let opened = signal(None::<Opened>);
        let origin = memo(move || {
            opened.get().map(|opened| match opened {
                Opened::Button => MenuOrigin::Anchor,
                Opened::Context(origin) => origin,
            })
        });
        let kept = memo(move || {
            let path = path.get();
            let focus = focus_key.get();
            entries.with(|entries| kept_levels(entries, &path, focus.as_ref()))
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
            typed: var(Typeahead::default()),
            surface_rects: signal(Vec::new()),
            press: (*kivact_core::style::use_theme().get().ext::<MenuPress>())
                .0
                .then(Press::new),
        };
        // A closed menu forgets its submenus, its focus, any request and
        // where it was asked to appear.
        effect(move || {
            if !open.get() {
                if let Some(press) = this.press {
                    press.cancel_open();
                }
                path.set(Vec::new());
                want.set(None);
                focus_key.set(None);
                opened.set(None);
            }
        });
        // A request for a level that is not open is dropped.
        effect(move || {
            let open_levels = path.with(Vec::len) + 1;
            if want.with(|want| {
                want.as_ref()
                    .is_some_and(|want| want.depth() >= open_levels)
            }) {
                want.set(None);
            }
        });
        // Bring the open levels in line with what the entries keep: a submenu
        // whose item left (or was disabled) closes and focus goes back to its
        // parent item; so does a level whose focused item left, and at the top
        // level that closes the whole menu.
        effect(move || {
            let kept = kept.get();
            if !open.get() {
                return;
            }
            if kept == 0 {
                this.close(PopupCloseReason::Escape);
                return;
            }
            if kept < path.with_untracked(Vec::len) + 1 {
                let parent = path.with_untracked(|path| path.get(kept - 1).cloned());
                let held = parent.filter(|key| {
                    entries.with_untracked(|entries| {
                        path.with_untracked(|path| item_enabled(entries, &path[..kept - 1], key))
                    })
                });
                path.update(|path| path.truncate(kept - 1));
                focus_key.set(None);
                match held {
                    Some(key) => this.request_key(kept - 1, key),
                    None => this.request(kept - 1, Edge::First),
                }
            }
        });
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
        self.opened.set(Some(Opened::Button));
        self.popup.open_binding().set(true);
    }

    /// Open as a context menu; the focus later returns to the element that
    /// has it now.
    pub(super) fn open_at(&self, origin: MenuOrigin) {
        self.opened.set(Some(Opened::Context(origin)));
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
        self.want.set(Some(Want::Command {
            depth,
            target: Target::Edge(edge),
        }));
    }

    /// Ask for the focus on `key` of the level at `depth`; a pointer never
    /// replaces it.
    pub(super) fn request_key(&self, depth: usize, key: K) {
        self.want.set(Some(Want::Command {
            depth,
            target: Target::Key(key),
        }));
    }

    /// The pointer rests on `key` of the level at `depth`: asked for unless a
    /// command is waiting.
    pub(super) fn hover(&self, depth: usize, key: K) {
        self.want.update(|want| {
            if !matches!(want, Some(Want::Command { .. })) {
                *want = Some(Want::Hover { depth, key });
            }
        });
    }

    /// Open the submenu of `key` (an item of the level at `depth`) without
    /// moving focus.
    pub(super) fn open_hover(&self, depth: usize, key: K) {
        let same = self
            .path
            .with_untracked(|path| path.len() == depth + 1 && path.get(depth) == Some(&key));
        if !same {
            self.path.update(|path| {
                path.truncate(depth);
                path.push(key);
            });
        }
    }

    /// Open the submenu of `key` and focus its first enabled item.
    pub(super) fn open_submenu(&self, depth: usize, key: K) {
        self.open_hover(depth, key);
        self.request(depth + 1, Edge::First);
    }

    /// Close every level deeper than `depth`.
    pub(super) fn close_below(&self, depth: usize) {
        if self.path.with_untracked(|path| path.len() > depth) {
            self.path.update(|path| path.truncate(depth));
        }
    }

    /// Close the level at `depth` (not the top level) and focus the item that
    /// opened it.
    pub(super) fn close_level(&self, depth: usize) {
        let parent = self.path.with(|path| path.get(depth - 1).cloned());
        self.path.update(|path| path.truncate(depth - 1));
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
