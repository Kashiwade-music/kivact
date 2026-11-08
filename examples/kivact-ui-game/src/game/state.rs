//! Role: Shared signal handles for Game catalog specimen commands.

use kivact::state::Signal;

#[derive(Clone, Copy)]
pub(super) struct Demo {
    // xtask-allow(state-fields): Caller-owned open binding for the independent create dialog.
    pub(super) create: Signal<bool>,
    // xtask-allow(state-fields): Caller-owned open binding for the independent delete dialog.
    pub(super) delete: Signal<bool>,
    // xtask-allow(state-fields): Caller-owned open binding for the independent details drawer.
    pub(super) drawer: Signal<bool>,
    // xtask-allow(state-fields): Caller-owned open binding for the independent command palette.
    pub(super) palette: Signal<bool>,
    pub(super) rows: Signal<Vec<Record>>,
    pub(super) selected: Signal<Vec<String>>,
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
