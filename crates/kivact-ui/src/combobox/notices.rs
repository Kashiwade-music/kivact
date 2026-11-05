//! Role: Announce accepted multiple-selection transactions without mirroring selection state.
use kivact_core::state::{ItemKey, Signal, Value, signal};
use kivact_core::view::{LiveRegion, View};
use kivact_interaction::{Announcer, ListItem, announcer};

#[derive(Clone, Copy)]
pub(super) struct Notices {
    outlet: Announcer,
    label: Signal<String>,
    added: Value<String>,
    removed: Value<String>,
    cleared: Value<String>,
}
impl Notices {
    pub(super) fn new() -> Self {
        let label = signal(String::new());
        Self {
            outlet: announcer(),
            label,
            added: kivact_core::i18n::message_with(
                "combobox.token_added",
                "Added {label}",
                move || [("label", label.get())],
            ),
            removed: kivact_core::i18n::message_with(
                "combobox.token_removed",
                "Removed {label}",
                move || [("label", label.get())],
            ),
            cleared: kivact_core::i18n::message("combobox.all_removed", "All items removed"),
        }
    }
    pub(super) fn accepted<K: ItemKey>(self, before: &[K], after: &[K], labels: &[ListItem<K>]) {
        if after.is_empty() && before.len() > 1 {
            self.outlet
                .announce(self.cleared.get_untracked(), LiveRegion::Polite);
            return;
        }
        let change = after
            .iter()
            .find(|key| !before.contains(key))
            .map(|key| (key, self.added))
            .or_else(|| {
                before
                    .iter()
                    .find(|key| !after.contains(key))
                    .map(|key| (key, self.removed))
            });
        if let Some((key, message)) = change {
            self.label.set(
                labels
                    .iter()
                    .find(|row| row.key() == key)
                    .map(ListItem::label)
                    .unwrap_or_default(),
            );
            self.outlet
                .announce(message.get_untracked(), LiveRegion::Polite);
        }
    }
    pub(super) fn outlet(self) -> View {
        self.outlet.outlet()
    }
}
