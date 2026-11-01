//! Role: Shared signal handles for Game catalog specimen commands.

use kivact::Signal;

#[derive(Clone, Copy)]
pub(super) struct Demo {
    pub(super) create: Signal<bool>,
    pub(super) delete: Signal<bool>,
    pub(super) drawer: Signal<bool>,
    pub(super) palette: Signal<bool>,
    pub(super) rows: Signal<Vec<Record>>,
    pub(super) selected: Signal<Vec<String>>,
}

/// One work order of the table specimen.
#[derive(Clone, PartialEq)]
pub(super) struct Record {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) description: String,
    pub(super) owner: String,
    pub(super) status: String,
    pub(super) tone: kivact::ui::Tone,
    pub(super) priority: u8,
    pub(super) progress: f64,
    pub(super) updated: String,
}

impl Record {
    /// Approve the order: it passes and is complete.
    pub(super) fn approve(&mut self) {
        self.status = "PASSED".into();
        self.tone = kivact::ui::Tone::Success;
        self.progress = 100.;
    }
}
