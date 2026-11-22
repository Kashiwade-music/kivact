//! Role: The shared signal handles of a catalog page: its overlay and its records.

use super::record::Record;
use super::records::records;
use kivact::state::{Signal, binding, signal};
/// The one overlay the catalog shows; opening another replaces the open one.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Overlay {
    None,
    Create,
    Delete,
    Drawer,
    Palette,
}

// xtask-allow(state): record: three shared catalog signals with different writers: overlay (the one open overlay, via show, hide, toggle and open), rows (the approve, reassign and delete handlers) and selected (the table selection, cleared by delete)
#[derive(Clone, Copy)]
pub(crate) struct Demo {
    pub(crate) overlay: Signal<Overlay>,
    pub(crate) rows: Signal<Vec<Record>>,
    pub(crate) selected: Signal<Vec<String>>,
}

impl Demo {
    /// The overlay's open binding, for a dialog, drawer or command palette
    /// (API-13: a component that is meaningless while closed takes `open`).
    pub(crate) fn open(self, which: Overlay) -> Signal<bool> {
        let overlay = self.overlay;
        binding(
            move || overlay.get() == which,
            move |open| {
                if open {
                    overlay.set(which);
                } else if overlay.get_untracked() == which {
                    overlay.set(Overlay::None);
                }
            },
        )
    }

    /// Show `which`, closing the overlay that was open.
    pub(crate) fn show(self, which: Overlay) {
        self.overlay.set(which);
    }

    /// Close `which` when it is the open overlay.
    pub(crate) fn hide(self, which: Overlay) {
        if self.overlay.get_untracked() == which {
            self.overlay.set(Overlay::None);
        }
    }
}

impl Demo {
    /// A page's own demo state: no overlay open, the fixture records, nothing selected.
    pub(crate) fn new() -> Demo {
        Demo {
            overlay: signal(Overlay::None),
            rows: signal(records()),
            selected: signal(Vec::new()),
        }
    }
}
