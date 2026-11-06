//! Role: Dock text resolution and shared layout editing operations.

use super::data::{DockCx, Shared, Texts};
use super::model::{DockLayout, DockSide, DockTab};
use kivact::geometry::Rect;
use kivact::i18n::message;
use kivact::interaction::{ListCommandModifier, ListRangeOptions};
use kivact::state::Value;
use kivact::view::{Handled, Key, KeyEvent, Modifiers};
use std::rc::Rc;

impl Texts {
    #[track_caller]
    pub(super) fn new() -> Self {
        Self {
            rename: message("creative.dock.rename", "Rename panel"),
            close_panel: message("creative.dock.close_panel", "Close Panel"),
            close_others: message("creative.dock.close_others", "Close Other Panels"),
            close_container: message("creative.dock.close_container", "Close Container"),
            maximize: message("creative.dock.maximize", "Maximize Panel"),
            restore: message("creative.dock.restore", "Restore Panel"),
            maximize_shortcut: message("creative.dock.maximize_shortcut", "Ctrl+Space"),
            move_left: message("creative.dock.move_left", "Move Tab Left"),
            move_right: message("creative.dock.move_right", "Move Tab Right"),
            split: [
                message("creative.dock.split_left", "Split Left"),
                message("creative.dock.split_right", "Split Right"),
                message("creative.dock.split_top", "Split Top"),
                message("creative.dock.split_bottom", "Split Bottom"),
            ],
            move_to: message("creative.dock.move_to", "Move To"),
            edge: message("creative.dock.edge", "Dock Edge"),
            into_tabs: message("creative.dock.into_tabs", "Into Tabs"),
            side: [
                message("creative.dock.left", "Left"),
                message("creative.dock.right", "Right"),
                message("creative.dock.top", "Top"),
                message("creative.dock.bottom", "Bottom"),
            ],
        }
    }
}

fn side_index(side: DockSide) -> usize {
    match side {
        DockSide::Left => 0,
        DockSide::Right => 1,
        DockSide::Top => 2,
        DockSide::Bottom => 3,
    }
}

impl Texts {
    pub(super) fn split(self, side: DockSide) -> Value<String> {
        self.split[side_index(side)]
    }
    pub(super) fn side(self, side: DockSide) -> Value<String> {
        self.side[side_index(side)]
    }
}

impl<K: kivact::state::ItemKey> DockCx<K> {
    pub(super) fn shared(self) -> Rc<Shared<K>> {
        self.shared.with(Rc::clone)
    }
    /// Apply an edit to the normalized layout and write it when it changed.
    pub(super) fn edit(self, edit: impl FnOnce(&mut DockLayout<K>) -> bool) -> bool {
        let mut next = self.current.get_untracked();
        if edit(&mut next) {
            self.layout.set(next);
            true
        } else {
            false
        }
    }
    /// Run a close command and notify each removed tab.
    pub(super) fn close(self, edit: impl FnOnce(&mut DockLayout<K>) -> Vec<DockTab<K>>) {
        let mut removed = Vec::new();
        self.edit(|layout| {
            removed = edit(layout);
            !removed.is_empty()
        });
        if let Some(callback) = self.shared().on_close {
            for tab in removed {
                callback.call(tab.id);
            }
        }
    }
    pub(super) fn toggle_maximize(self, id: &K) {
        self.edit(|layout| {
            if layout
                .maximized_container()
                .is_some_and(|tabs| tabs.contains(id))
            {
                layout.restore()
            } else {
                layout.maximize(id)
            }
        });
    }
    /// Register a tab's reported window rectangle for strip drop targets.
    pub(super) fn register_rect(self, id: K, rect: Value<Option<Rect>>) {
        let shared = self.shared();
        let token = shared.next_rect.get().wrapping_add(1);
        shared.next_rect.set(token);
        shared.rects.borrow_mut().insert(id.clone(), (token, rect));
        kivact::state::on_cleanup(move || {
            let mut rects = shared.rects.borrow_mut();
            if rects.get(&id).is_some_and(|(owner, _)| *owner == token) {
                rects.remove(&id);
            }
        });
    }
    pub(super) fn tab_rect(self, id: &K) -> Option<Rect> {
        let rect = self
            .shared()
            .rects
            .borrow()
            .get(id)
            .map(|(_, rect)| *rect)?;
        rect.get_untracked()
    }
    /// Ctrl or Cmd+Space toggles maximization of the container showing `id`.
    pub(super) fn maximize_key(self, id: &K, event: &KeyEvent) -> Handled {
        if event.key() == Key::Space && command_held(event.modifiers()) {
            self.toggle_maximize(id);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

/// Whether the application command modifier is held, as selections read it
/// (Ctrl or Cmd).
fn command_held(modifiers: Modifiers) -> bool {
    match ListRangeOptions::default().command {
        ListCommandModifier::Control => modifiers.ctrl,
        ListCommandModifier::Meta => modifiers.meta,
        ListCommandModifier::ControlOrMeta => modifiers.ctrl || modifiers.meta,
    }
}
