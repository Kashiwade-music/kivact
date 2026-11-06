//! Role: Guard the undelivered native edits without borrowing the view tree.

use std::cell::RefCell;
use std::rc::Rc;

use kivact_engine::contract::HostEvent;

use crate::view::tree::{TextEditsInFlight, ViewNodes};

/// Counts native edits already applied by the host until their delivery ends.
#[must_use = "the counts are forgotten when the guard drops"]
pub(crate) struct TextEditsAhead {
    counts: Rc<RefCell<TextEditsInFlight>>,
}

impl TextEditsAhead {
    /// Begin delivery while borrowing the tree only to acquire its count store.
    pub(crate) fn begin(tree: &mut ViewNodes, events: &[HostEvent]) -> Self {
        let counts = tree.future_text_edits.clone();
        for event in events {
            if let HostEvent::TextChanged { node, .. } = event {
                *counts.borrow_mut().0.entry(*node).or_default() += 1;
            }
        }
        Self { counts }
    }

    /// Mark one native edit as delivered before its handlers run.
    pub(crate) fn deliver(&self, event: &HostEvent) {
        if let HostEvent::TextChanged { node, .. } = event {
            let mut counts = self.counts.borrow_mut();
            if let Some(count) = counts.0.get_mut(node) {
                *count -= 1;
                if *count == 0 {
                    counts.0.remove(node);
                }
            }
        }
    }
}
impl Drop for TextEditsAhead {
    fn drop(&mut self) {
        self.counts.borrow_mut().0.clear();
    }
}
