//! Role: The GPU export pools the window has made, and pending engine notices.

use kivact_types::ids::DrawableId;
use kivact_types::render::ExportLoss;
use std::collections::HashMap;

/// A change of the GPU export pools that the engine must act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportNotice {
    /// The pool of this request is gone (device lost, GPU given up).
    Lost(ExportLoss),
    /// A GPU device became usable for exports (start finished, or a new
    /// device after a loss).
    Available,
}

/// The GPU export pools the window has made, and what the engine has not yet
/// been told. Lives as long as the presenter (outlives any GPU context).
#[derive(Default)]
pub(crate) struct ExportRegistry {
    /// The generation of the live pool of each sink.
    pools: HashMap<DrawableId, u64>,
    /// Notices not yet taken by the engine.
    notices: Vec<ExportNotice>,
}

impl ExportRegistry {
    /// A pool of `generation` was made for `sink`.
    pub(crate) fn made(&mut self, sink: DrawableId, generation: u64) {
        self.pools.insert(sink, generation);
    }

    /// The pool of `sink` was dropped on request; no notice.
    pub(crate) fn dropped(&mut self, sink: DrawableId) {
        self.pools.remove(&sink);
    }

    /// The GPU context went away: every live pool is lost.
    pub(crate) fn lost_all(&mut self) {
        let mut lost: Vec<(DrawableId, u64)> = self.pools.drain().collect();
        lost.sort_by_key(|(sink, _)| *sink);
        self.notices.extend(
            lost.into_iter()
                .map(|(sink, generation)| ExportNotice::Lost(ExportLoss { sink, generation })),
        );
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
