//! Role: Slot-indexed measurement and computed-text caches, independent of topology.
//!
//! The text column holds each node's resolved inherited text style (the
//! computed style): derived from the `Text` props of the node and its
//! ancestors, filled on the first read of a node and cleared when one of
//! them may have changed (a `TextStyle` write on the node or an ancestor, a
//! move to another parent, a reused slot). The entry is shared (`Rc`), so
//! a reader keeps it without copying the style's family and features.
//!
//! The leaf column holds what a node's own kind measures: an image's
//! natural size or a text's shaped extents (one per kind, which never
//! changes after the node is made).
use kivact_types::{ComputedTextStyle, Size};
use std::cell::{Cell, RefCell, RefMut};
use std::rc::Rc;

/// What a node's leaf measures: one per node kind, which never changes.
#[derive(Clone, Debug)]
pub(crate) enum LeafMeasure {
    /// Neither an image nor a text: an intrinsic size or a leaf callback
    /// measures it.
    None,
    /// An image's natural size, once its asset is loaded.
    Natural(Cell<Option<Size>>),
    /// A text's shaped extents, kept while its style and text hold.
    Text(Box<RefCell<Option<Box<TextExtents>>>>),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MeasureColumns {
    /// What the node at each slot measures (see [`LeafMeasure`]).
    pub(crate) leaf: Vec<LeafMeasure>,
    computed_text: Vec<RefCell<Option<Rc<ComputedTextStyle>>>>,
}
impl MeasureColumns {
    pub(crate) fn push(&mut self, kind: &crate::contract::NodeKind) {
        self.leaf.push(leaf_for(kind));
        self.computed_text.push(RefCell::new(None));
    }

    /// Measure the node at `slot` one way only: an editor measures itself
    /// (`LeafMeasure::None`), else the node's kind does.
    pub(crate) fn set_measure(
        &mut self,
        slot: usize,
        kind: &crate::contract::NodeKind,
        editor: bool,
    ) {
        let leaf = if editor {
            LeafMeasure::None
        } else {
            leaf_for(kind)
        };
        let same = matches!(
            (&self.leaf[slot], &leaf),
            (LeafMeasure::None, LeafMeasure::None)
                | (LeafMeasure::Natural(_), LeafMeasure::Natural(_))
                | (LeafMeasure::Text(_), LeafMeasure::Text(_))
        );
        if !same {
            self.leaf[slot] = leaf;
        }
    }
    pub(crate) fn take_slot(&mut self, slot: usize) -> Option<Box<TextExtents>> {
        let extents = match std::mem::replace(&mut self.leaf[slot], LeafMeasure::None) {
            LeafMeasure::Text(extents) => extents.into_inner(),
            _ => None,
        };
        self.computed_text[slot].get_mut().take();
        extents
    }

    /// The natural size of the image at `slot`, once it is recorded.
    pub(crate) fn natural(&self, slot: usize) -> Option<Size> {
        match &self.leaf[slot] {
            LeafMeasure::Natural(natural) => natural.get(),
            _ => None,
        }
    }

    /// Keep `size` as the natural size of the image at `slot`.
    pub(crate) fn set_natural(&self, slot: usize, size: Option<Size>) {
        if let LeafMeasure::Natural(natural) = &self.leaf[slot] {
            natural.set(size);
        }
    }

    /// The shaped extents of the text at `slot`.
    pub(crate) fn extents(&self, slot: usize) -> RefMut<'_, Option<Box<TextExtents>>> {
        match &self.leaf[slot] {
            LeafMeasure::Text(extents) => extents.borrow_mut(),
            _ => panic!("invariant: a text node's leaf keeps its extents"),
        }
    }

    /// Forget the shaped extents of every text.
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn forget_extents(&self) {
        for leaf in &self.leaf {
            if let LeafMeasure::Text(extents) = leaf {
                *extents.borrow_mut() = None;
            }
        }
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
/// The leaf measure a node of `kind` needs.
fn leaf_for(kind: &crate::contract::NodeKind) -> LeafMeasure {
    match kind {
        crate::contract::NodeKind::Image(_) => LeafMeasure::Natural(Cell::new(None)),
        crate::contract::NodeKind::Text(_) => LeafMeasure::Text(Box::default()),
        _ => LeafMeasure::None,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TextExtents {
    pub(crate) style: kivact_types::ComputedTextStyle,
    pub(crate) single: Option<kivact_types::TextMetrics>,
    pub(crate) wrapped: Vec<(f64, Option<kivact_types::TextMetrics>)>,
}
