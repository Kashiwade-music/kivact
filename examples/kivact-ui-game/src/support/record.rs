//! Role: One fixture production order: its fields and its approval.

/// One production order of the table specimen.
#[derive(Clone, PartialEq)]
pub(crate) struct Record {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) owner: String,
    pub(crate) status: String,
    pub(crate) tone: kivact::interaction::Tone,
    pub(crate) priority: u8,
    pub(crate) progress: f64,
    pub(crate) updated: String,
}

impl Record {
    /// Approve the order: it passes and is complete.
    pub(crate) fn approve(&mut self) {
        self.status = "PASSED".into();
        self.tone = kivact::interaction::Tone::Success;
        self.progress = 100.;
    }
}
