//! Role: The text edits of a delivered batch that have not run yet.

use kivact_engine::contract::HostEvent;
use kivact_runtime::Runtime;

use crate::view::state::ViewState;

/// The `TextChanged` events of the batch being delivered that have not
/// run yet, per node: while one waits for a node, a controlled write of
/// its value is stale and dropped (`Tree::update`). Dropping it forgets
/// what is left, so the next batch counts anew.
#[must_use = "the counts are forgotten when the guard drops"]
pub(crate) struct TextEditsAhead<'a> {
    rt: &'a Runtime,
}

impl<'a> TextEditsAhead<'a> {
    /// Count the `TextChanged` events of `events` per node.
    pub(crate) fn count(rt: &'a Runtime, events: &[HostEvent]) -> Self {
        ViewState::tree_mut(rt).count_text_edits(events.iter().filter_map(|event| match event {
            HostEvent::TextChanged { node, .. } => Some(*node),
            _ => None,
        }));
        TextEditsAhead { rt }
    }

    /// `event` runs now: a `TextChanged` no longer waits for its node.
    pub(crate) fn deliver(&self, event: &HostEvent) {
        if let HostEvent::TextChanged { node, .. } = event {
            ViewState::tree_mut(self.rt).consume_text_edit(*node);
        }
    }
}

impl Drop for TextEditsAhead<'_> {
    fn drop(&mut self) {
        ViewState::tree_mut(self.rt).clear_text_edits();
    }
}
