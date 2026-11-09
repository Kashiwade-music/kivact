//! Role: Shared signal handles for Game catalog specimen commands.

use kivact::state::{Binding, Signal, binding};

/// The one overlay the catalog shows; opening another replaces the open one.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Overlay {
    None,
    Create,
    Delete,
    Drawer,
    Palette,
}

#[derive(Clone, Copy)]
pub(super) struct Demo {
    pub(super) overlay: Signal<Overlay>,
    pub(super) rows: Signal<Vec<Record>>,
    pub(super) selected: Signal<Vec<String>>,
}

impl Demo {
    /// The overlay's open binding, for a dialog, drawer or command palette
    /// (ARG-4: a component that is meaningless while closed takes `open`).
    pub(super) fn open(self, which: Overlay) -> Binding<bool> {
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
    pub(super) fn show(self, which: Overlay) {
        self.overlay.set(which);
    }

    /// Close `which` when it is the open overlay.
    pub(super) fn hide(self, which: Overlay) {
        if self.overlay.get_untracked() == which {
            self.overlay.set(Overlay::None);
        }
    }

    /// Open `which` when another overlay is open, close it when it is open.
    pub(super) fn toggle(self, which: Overlay) {
        self.overlay
            .update(|open| *open = if *open == which { Overlay::None } else { which });
    }
}

/// One production order of the table specimen.
#[derive(Clone, PartialEq)]
pub(super) struct Record {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) description: String,
    pub(super) owner: String,
    pub(super) status: String,
    pub(super) tone: kivact::interaction::Tone,
    pub(super) priority: u8,
    pub(super) progress: f64,
    pub(super) updated: String,
}

impl Record {
    /// Approve the order: it passes and is complete.
    pub(super) fn approve(&mut self) {
        self.status = "PASSED".into();
        self.tone = kivact::interaction::Tone::Success;
        self.progress = 100.;
    }
}
