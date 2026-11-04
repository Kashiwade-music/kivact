//! Role: Describe a keyed collection item moving to a new position.

/// A keyed item moving to a new position in its collection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reorder<K> {
    pub key: K,
    pub to: usize,
}
