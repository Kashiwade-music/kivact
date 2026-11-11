//! Role: The links of one graph node (what it read, or who reads it): up to two inline.
//!
//! Most nodes have at most two links, so they are stored in the node itself;
//! a third link moves them to a heap vector.

use crate::reactive::arena::Key;

pub(crate) type Link = (Key, u32);

const INLINE: usize = 2;
const NONE: Link = (
    Key {
        slot: 0,
        generation: 0,
    },
    0,
);

/// A small vector of links: `INLINE` stored in place, more on the heap.
pub(crate) enum Links {
    Inline { len: u8, items: [Link; INLINE] },
    Heap(Vec<Link>),
}

impl Default for Links {
    fn default() -> Self {
        Links::Inline {
            len: 0,
            items: [NONE; INLINE],
        }
    }
}

impl std::ops::Deref for Links {
    type Target = [Link];

    fn deref(&self) -> &[Link] {
        match self {
            Links::Inline { len, items } => &items[..usize::from(*len)],
            Links::Heap(v) => v,
        }
    }
}

impl std::ops::DerefMut for Links {
    fn deref_mut(&mut self) -> &mut [Link] {
        match self {
            Links::Inline { len, items } => &mut items[..usize::from(*len)],
            Links::Heap(v) => v,
        }
    }
}

impl Links {
    pub(crate) fn push(&mut self, link: Link) {
        match self {
            Links::Inline { len, items } if usize::from(*len) < INLINE => {
                items[usize::from(*len)] = link;
                *len += 1;
            }
            Links::Inline { items, .. } => {
                let mut v = Vec::with_capacity(INLINE * 2);
                v.extend_from_slice(&items[..]);
                v.push(link);
                *self = Links::Heap(v);
            }
            Links::Heap(v) => v.push(link),
        }
    }

    pub(crate) fn swap_remove(&mut self, index: usize) -> Link {
        match self {
            Links::Inline { len, items } => {
                let last = usize::from(*len) - 1;
                items.swap(index, last);
                *len -= 1;
                items[last]
            }
            Links::Heap(v) => v.swap_remove(index),
        }
    }

    pub(crate) fn remove(&mut self, index: usize) -> Link {
        match self {
            Links::Inline { len, items } => {
                let n = usize::from(*len);
                let link = items[index];
                items.copy_within(index + 1..n, index);
                *len -= 1;
                link
            }
            Links::Heap(v) => v.remove(index),
        }
    }
}

/// Owning iterator over the links, without allocating for the inline ones.
pub(crate) enum LinkIter {
    Inline(std::iter::Take<std::array::IntoIter<Link, INLINE>>),
    Heap(std::vec::IntoIter<Link>),
}

impl Iterator for LinkIter {
    type Item = Link;

    fn next(&mut self) -> Option<Link> {
        match self {
            LinkIter::Inline(it) => it.next(),
            LinkIter::Heap(it) => it.next(),
        }
    }
}

impl IntoIterator for Links {
    type Item = Link;
    type IntoIter = LinkIter;

    fn into_iter(self) -> LinkIter {
        match self {
            Links::Inline { len, items } => {
                LinkIter::Inline(items.into_iter().take(usize::from(len)))
            }
            Links::Heap(v) => LinkIter::Heap(v.into_iter()),
        }
    }
}
