//! Role: The flame chart of recorded turns and frames.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

/// The stage and frame bands of the Performance panel.
pub(crate) fn flame_chart(
    turns: Signal<Vec<p::TurnGraph>>,
    frames: Signal<Vec<p::FrameInfo>>,
) -> impl IntoView {
    let _ = (turns, frames);
    node()
}
