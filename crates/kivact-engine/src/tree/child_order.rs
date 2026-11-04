//! Role: The sole child order, packed for layout and linked during structural writes.

use crate::tree::node::NodeIndex;
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct Link {
    previous: Option<NodeIndex>,
    next: Option<NodeIndex>,
}

#[derive(Clone, Debug, Default)]
struct Editing {
    // Empty while editing: capacity only, never a shadow child sequence.
    packed: Vec<NodeIndex>,
    links: HashMap<usize, Link>,
    first: Option<NodeIndex>,
    last: Option<NodeIndex>,
}

impl Editing {
    fn from_packed(mut packed: Vec<NodeIndex>) -> Self {
        let mut order = Self {
            links: HashMap::with_capacity(packed.len()),
            ..Self::default()
        };
        for child in packed.drain(..) {
            order.insert(child, None);
        }
        order.packed = packed;
        order
    }
    fn insert(&mut self, child: NodeIndex, before: Option<NodeIndex>) {
        let before = before.filter(|index| self.links.contains_key(&index.0));
        let previous = before.map_or(self.last, |index| self.links[&index.0].previous);
        debug_assert!(!self.links.contains_key(&child.0));
        self.links.insert(
            child.0,
            Link {
                previous,
                next: before,
            },
        );
        if let Some(previous) = previous {
            self.links
                .get_mut(&previous.0)
                .expect("invariant: previous sibling")
                .next = Some(child);
        } else {
            self.first = Some(child);
        }
        if let Some(before) = before {
            self.links
                .get_mut(&before.0)
                .expect("invariant: next sibling")
                .previous = Some(child);
        } else {
            self.last = Some(child);
        }
    }
    fn remove(&mut self, child: NodeIndex) {
        let Some(link) = self.links.remove(&child.0) else {
            return;
        };
        if let Some(previous) = link.previous {
            self.links
                .get_mut(&previous.0)
                .expect("invariant: previous sibling")
                .next = link.next;
        } else {
            self.first = link.next;
        }
        if let Some(next) = link.next {
            self.links
                .get_mut(&next.0)
                .expect("invariant: next sibling")
                .previous = link.previous;
        } else {
            self.last = link.previous;
        }
    }
    fn into_packed(mut self) -> Vec<NodeIndex> {
        let count = self.links.len();
        let mut at = self.first;
        while let Some(child) = at {
            let link = self
                .links
                .remove(&child.0)
                .expect("invariant: packed child exists");
            at = link.next;
            self.packed.push(child);
        }
        debug_assert_eq!(self.packed.len(), count);
        debug_assert!(self.links.is_empty());
        self.packed
    }
}

/// Exactly one authoritative order. Editing consumes the packed sequence;
/// packing consumes the editing links. It never owns both sequences at once.
#[derive(Clone, Debug)]
enum Order {
    Packed(Vec<NodeIndex>),
    Editing(Box<Editing>),
}
#[derive(Clone, Debug)]
pub(crate) struct ChildOrder(Order);
impl Default for ChildOrder {
    fn default() -> Self {
        Self(Order::Packed(Vec::new()))
    }
}
impl ChildOrder {
    pub(crate) fn len(&self) -> usize {
        match &self.0 {
            Order::Packed(v) => v.len(),
            Order::Editing(v) => v.links.len(),
        }
    }
    fn editing(&mut self) -> &mut Editing {
        if let Order::Packed(packed) = &mut self.0 {
            let packed = std::mem::take(packed);
            self.0 = Order::Editing(Box::new(Editing::from_packed(packed)));
        }
        let Order::Editing(order) = &mut self.0 else {
            unreachable!()
        };
        order
    }
    pub(crate) fn insert(&mut self, child: NodeIndex, before: Option<NodeIndex>) {
        if before.is_none()
            && let Order::Packed(packed) = &mut self.0
        {
            packed.push(child);
            return;
        }
        self.editing().insert(child, before);
    }
    pub(crate) fn remove(&mut self, child: NodeIndex) {
        if let Order::Packed(packed) = &mut self.0 {
            if packed.last() == Some(&child) {
                packed.pop();
                return;
            }
            if packed.is_empty() {
                return;
            }
        }
        self.editing().remove(child);
    }
    pub(crate) fn pack(&mut self) {
        if matches!(&self.0, Order::Editing(_)) {
            let Order::Editing(order) = std::mem::replace(&mut self.0, Order::Packed(Vec::new()))
            else {
                unreachable!()
            };
            self.0 = Order::Packed(order.into_packed());
        }
    }
    pub(crate) fn is_editing(&self) -> bool {
        matches!(&self.0, Order::Editing(_))
    }
    pub(crate) fn iter(&self) -> Children<'_> {
        let (first, last) = match &self.0 {
            Order::Packed(_) => (None, None),
            Order::Editing(order) => (order.first, order.last),
        };
        Children {
            order: self,
            range: 0..self.len(),
            first,
            last,
        }
    }
}

pub(crate) struct Children<'a> {
    order: &'a ChildOrder,
    range: std::ops::Range<usize>,
    first: Option<NodeIndex>,
    last: Option<NodeIndex>,
}
impl Iterator for Children<'_> {
    type Item = NodeIndex;
    fn next(&mut self) -> Option<NodeIndex> {
        let at = self.range.next()?;
        match &self.order.0 {
            Order::Packed(packed) => Some(packed[at]),
            Order::Editing(order) => {
                let child = self.first.expect("invariant: next child exists");
                self.first = order.links[&child.0].next;
                Some(child)
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}
impl DoubleEndedIterator for Children<'_> {
    fn next_back(&mut self) -> Option<NodeIndex> {
        let at = self.range.next_back()?;
        match &self.order.0 {
            Order::Packed(packed) => Some(packed[at]),
            Order::Editing(order) => {
                let child = self.last.expect("invariant: previous child exists");
                self.last = order.links[&child.0].previous;
                Some(child)
            }
        }
    }
}
impl ExactSizeIterator for Children<'_> {
    fn len(&self) -> usize {
        self.range.len()
    }
}
impl IntoIterator for ChildOrder {
    type Item = NodeIndex;
    type IntoIter = std::vec::IntoIter<NodeIndex>;
    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Order::Packed(packed) => packed,
            Order::Editing(order) => order.into_packed(),
        }
        .into_iter()
    }
}
