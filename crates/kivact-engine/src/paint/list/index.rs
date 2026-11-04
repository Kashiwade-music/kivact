//! Role: Child index of a container with many children for fast paint list rebuilds.
//!
//! The child index of a container with many children (see the parent
//! module's invariants): what each child can draw on, relative to its
//! layout location, and where it is, so a rebuild of the paint list finds
//! the children that reach the clip by binary search.
//!
//! Invariants:
//! - Entries follow the container's children in order, layers (overlays,
//!   fixed boxes) left out; a hidden child keeps an entry that reaches
//!   nothing. A sticky child's entry is outside the search (it may be
//!   stuck anywhere): [`ChildIndex::sticky`] lists them all.
//! - An entry's reach is the child's ink (`super::ink`: what its subtree
//!   can draw on) at its location, grown by [`SLACK`]; a child whose ink is
//!   not finite reaches everywhere.
//! - After a layout pass every entry takes its child's new location; only
//!   the children whose size or content size changed are looked at again
//!   (their ink). After a change in a child's subtree
//!   (`ChildIndex::touch`), that child is. The container's children
//!   changing drops the index (the parent module).

use crate::model::Model;
use crate::paint::state::{Axis, ChildIndex, Extent, Indexed};
use crate::tree::node::HostNode;
use kivact_layout::NodeLayout;
use kivact_types::{NodeId, Rect};

use super::ink::{grow, ink, overlaps};

/// Logical pixels an entry is grown by: the index computes a child's reach
/// relative to its location, the walk in window space, and the two may
/// round apart (far less than the margin, so no pixel depends on it).
const SLACK: f32 = 0.01;

impl Indexed {
    fn new(
        host: &Model,
        id: NodeId,
        n: Option<&HostNode>,
        layout: NodeLayout,
        generation: u64,
        order: usize,
    ) -> Self {
        let extent = match n.and_then(|n| ink(host, id, n, &layout, generation)) {
            Some(r) if [r.x, r.y, r.width, r.height].iter().all(|v| v.is_finite()) => {
                Extent::Within(grow(r, SLACK))
            }
            Some(_) => Extent::Anywhere,
            None => Extent::Nothing,
        };
        Indexed {
            id,
            order,
            sticky: n.is_some_and(|n| n.place.sticky),
            location: layout.location,
            size: layout.size,
            content: layout.content_size,
            extent,
        }
    }

    /// Its reach in the container's space (`None`: everywhere), searched
    /// (a sticky child's is not: it reaches nothing here).
    fn reach(&self) -> Option<Option<Rect>> {
        if self.sticky {
            return None;
        }
        match self.extent {
            Extent::Nothing => None,
            Extent::Anywhere => Some(None),
            Extent::Within(r) => Some(Some(Rect {
                x: r.x + self.location.x,
                y: r.y + self.location.y,
                ..r
            })),
        }
    }
}

impl Axis {
    /// The `(start, end)` of `r` along the axis.
    fn span(self, r: &Rect) -> (f32, f32) {
        match self {
            Axis::X => (r.x, r.x + r.width),
            Axis::Y => (r.y, r.y + r.height),
        }
    }
}

impl ChildIndex {
    /// The index of the children `children`, laid out by layout pass
    /// `pass`, with inks of generation `generation`.
    pub(super) fn build(
        host: &Model,
        children: impl ExactSizeIterator<Item = NodeId>,
        pass: u64,
        generation: u64,
    ) -> Self {
        let layouts = host;
        let entries = children
            .filter_map(|c| Some((c, host.node(c)?)))
            .filter(|(_, n)| !n.is_layer())
            .enumerate()
            .map(|(order, (c, n))| {
                Indexed::new(
                    host,
                    c,
                    Some(n),
                    layouts.computed_layout(c).unwrap_or_default(),
                    generation,
                    order,
                )
            })
            .collect();
        let mut index = ChildIndex {
            entries,
            axis: None,
            starts: Vec::new(),
            furthest: Vec::new(),
            sticky: Vec::new(),
            pass,
            touched: Vec::new(),
        };
        index.sort();
        index
    }

    /// Something in the subtree of the child `id` changed (its ink).
    pub(super) fn touch(&mut self, id: NodeId) {
        self.touched.push(id);
    }

    /// Bring the entries up to layout pass `pass` and the touched children
    /// (inks of generation `generation`).
    pub(super) fn refresh(&mut self, host: &Model, pass: u64, generation: u64) {
        let relaid = pass != self.pass;
        if !relaid && self.touched.is_empty() {
            return;
        }
        let mut touched = std::mem::take(&mut self.touched);
        touched.sort_unstable();
        let layouts = host;
        for e in &mut self.entries {
            let again = touched.binary_search(&e.id).is_ok();
            if !relaid && !again {
                continue;
            }
            let layout = layouts.computed_layout(e.id).unwrap_or_default();
            if again || layout.size != e.size || layout.content_size != e.content {
                *e = Indexed::new(host, e.id, host.node(e.id), layout, generation, e.order);
            } else {
                e.location = layout.location;
            }
        }
        self.pass = pass;
        self.sort();
    }

    /// Find the axis the reaches are sorted along, the search arrays and
    /// the sticky entries.
    fn sort(&mut self) {
        self.sticky.clear();
        self.sticky.extend(
            self.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.sticky)
                .map(|(i, _)| i),
        );
        self.axis = [Axis::Y, Axis::X].into_iter().find(|axis| {
            let mut last = f32::NEG_INFINITY;
            self.entries.iter().all(|e| match e.reach() {
                None => true,
                Some(None) => false,
                Some(Some(r)) => {
                    let start = axis.span(&r).0;
                    let sorted = start >= last;
                    last = start;
                    sorted
                }
            })
        });
        self.starts.clear();
        self.furthest.clear();
        if let Some(axis) = self.axis {
            let (mut start, mut end) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
            for e in &self.entries {
                if let Some(Some(r)) = e.reach() {
                    let span = axis.span(&r);
                    start = span.0;
                    end = end.max(span.1);
                }
                self.starts.push(start);
                self.furthest.push(end);
            }
        }
    }

    /// The sticky children (outside the search), in order.
    pub(super) fn sticky(&self) -> impl Iterator<Item = &Indexed> {
        self.sticky.iter().map(|i| &self.entries[*i])
    }

    /// The children (not sticky) whose reach meets `clip` (in the index's
    /// space), in order.
    pub(super) fn reaching(&self, clip: Rect) -> impl Iterator<Item = &Indexed> {
        let range = match self.axis {
            Some(axis) => {
                let (start, end) = axis.span(&clip);
                // Before `first` every reach ends at or before the clip
                // starts; from `last` on every reach starts at or after it
                // ends.
                let first = self.furthest.partition_point(|f| *f <= start);
                let last = self.starts.partition_point(|s| *s < end);
                first..last.max(first)
            }
            None => 0..self.entries.len(),
        };
        self.entries[range].iter().filter(move |e| match e.reach() {
            None => false,
            Some(None) => true,
            Some(Some(r)) => overlaps(r, clip),
        })
    }
}
