//! Role: Project shared node observations into public scroll and image values.

use super::node_ref::NodeRef;
use super::tree::ScrollState as ScrollReport;
use kivact_runtime::{Value, memo, require_setup};
pub use kivact_types::ImageStatus;

/// Position within the scrollable extent across the node's scrolling axes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollState {
    /// No axis has a scrollable extent, including an unbound reference.
    #[default]
    NotScrollable,
    /// Every scrollable axis is at its start.
    AtStart,
    /// At least one scrollable axis is between the combined endpoints.
    Middle,
    /// Every scrollable axis is at its end.
    AtEnd,
}

pub(crate) fn classify_scroll(report: ScrollReport) -> ScrollState {
    let extent = [
        (
            report.offset.x,
            (report.content.width - report.viewport.width).max(0.0),
        ),
        (
            report.offset.y,
            (report.content.height - report.viewport.height).max(0.0),
        ),
    ];
    let axes: Vec<_> = extent.into_iter().filter(|(_, max)| *max > 0.0).collect();
    if axes.is_empty() {
        ScrollState::NotScrollable
    } else if axes.iter().all(|(offset, _)| *offset <= 0.0) {
        ScrollState::AtStart
    } else if axes.iter().all(|(offset, max)| *offset >= *max) {
        ScrollState::AtEnd
    } else {
        ScrollState::Middle
    }
}

impl NodeRef {
    /// Reactive position in the scroll extent; unbound refs are not scrollable.
    /// Setup only.
    #[track_caller]
    pub fn scroll_state(&self) -> Value<ScrollState> {
        require_setup("NodeRef::scroll_state");
        // Keep host geometry reports active even when scrolling is disabled.
        let _ = self.rect();
        let node = *self;
        memo(move || {
            node.observed_status(|observation| observation.scroll_state.get())
                .unwrap_or_default()
        })
    }

    /// Reactive result of the current image source; unbound refs are loading.
    /// Setup only.
    #[track_caller]
    pub fn image_status(&self) -> Value<ImageStatus> {
        require_setup("NodeRef::image_status");
        let node = *self;
        memo(move || {
            node.observed_status(|observation| observation.image_status.get())
                .unwrap_or_default()
        })
    }
}
