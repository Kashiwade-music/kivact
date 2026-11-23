//! Role: The GPU export pools the window has made, and pending engine notices.

use arvane_protocol::ExportLoss;

/// A change of the GPU export pools that the engine must act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportNotice {
    /// The pool of this request is gone (device lost, GPU given up).
    Lost(ExportLoss),
    /// A GPU device became usable for exports (start finished, or a new
    /// device after a loss).
    Available,
}

/// What the engine has not yet been told about the window's GPU export pools.
/// The pools themselves live in the window's GPU context.
#[derive(Default)]
pub(crate) struct ExportRegistry {
    /// Notices not yet taken by the engine.
    notices: Vec<ExportNotice>,
}

impl ExportRegistry {
    /// The GPU context went away: its pools are lost.
    pub(crate) fn lost(&mut self, losses: Vec<ExportLoss>) {
        self.notices
            .extend(losses.into_iter().map(ExportNotice::Lost));
    }

    /// A GPU device became ready for exports.
    pub(crate) fn available(&mut self) {
        self.notices.push(ExportNotice::Available);
    }

    /// The notices since the last call, in order.
    pub(crate) fn take(&mut self) -> Vec<ExportNotice> {
        std::mem::take(&mut self.notices)
    }
}
