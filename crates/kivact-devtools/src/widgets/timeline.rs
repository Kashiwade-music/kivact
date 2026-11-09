//! Role: The timeline of recorded turns and frames.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

/// The time lanes of the Updates panel.
pub(crate) fn timeline(
    turns: Signal<Vec<p::TurnGraph>>,
    frames: Signal<Vec<p::FrameInfo>>,
    selected: Signal<Option<u64>>,
) -> impl IntoView {
    let _ = (turns, frames, selected);
    node()
}
