//! Role: Project shared node observations into public scroll and image values.

use super::node_ref::NodeRef;
use super::node_status_data::ScrollState;
use kivact_runtime::{Value, memo, require_setup};
use kivact_types::ImageStatus;

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
