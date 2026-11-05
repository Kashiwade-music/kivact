//! Role: Slot-indexed measurement and computed-text caches, independent of topology.
//!
//! The text column holds each node's resolved inherited text style (the
//! computed style): derived from the `Text` props of the node and its
//! ancestors, filled on the first read of a node and cleared when one of
//! them may have changed (a `TextStyle` write on the node or an ancestor, a
//! move to another parent, a reused slot). The entry is shared (`Rc`), so
//! a reader keeps it without copying the style's family and features.
use kivact_types::{ComputedTextStyle, Size};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// xtask-allow(state-fields): Slot-indexed natural size, extents and inherited text are independent rebuildable caches (S-9).
#[derive(Clone, Debug, Default)]
pub(crate) struct MeasureColumns {
    pub(crate) natural: Vec<Cell<Option<Size>>>,
    pub(crate) extents: Vec<RefCell<Option<Box<TextExtents>>>>,
    computed_text: Vec<RefCell<Option<Rc<ComputedTextStyle>>>>,
}
impl MeasureColumns {
    pub(crate) fn push(&mut self) {
        self.natural.push(Cell::new(None));
        self.extents.push(RefCell::new(None));
        self.computed_text.push(RefCell::new(None));
    }
    pub(crate) fn take_slot(&mut self, slot: usize) -> Option<Box<TextExtents>> {
        self.natural[slot].set(None);
        self.computed_text[slot].get_mut().take();
        self.extents[slot].get_mut().take()
    }

    /// The resolved text style of `slot`, if one is kept.
    pub(crate) fn computed_text(&self, slot: usize) -> Option<Rc<ComputedTextStyle>> {
        self.computed_text[slot].borrow().clone()
    }

    /// Keep `text` as the resolved text style of `slot`.
    pub(crate) fn set_computed_text(&self, slot: usize, text: Rc<ComputedTextStyle>) {
        *self.computed_text[slot].borrow_mut() = Some(text);
    }

    /// The text style of `slot` is resolved again at its next read.
    pub(crate) fn forget_computed_text(&mut self, slot: usize) {
        self.computed_text[slot].get_mut().take();
    }
}
// xtask-allow(state-fields): Single-line and width-keyed wrapped metrics are independent measurement caches (S-9).
#[derive(Clone, Debug)]
pub(crate) struct TextExtents {
    pub(crate) style: kivact_types::ComputedTextStyle,
    pub(crate) single: Option<kivact_types::TextMetrics>,
    pub(crate) wrapped: Vec<(f64, Option<kivact_types::TextMetrics>)>,
}
