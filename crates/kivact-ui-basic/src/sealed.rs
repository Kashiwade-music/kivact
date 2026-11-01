//! Role: Public-in-bounds traits that only this crate can name or implement.

/// Seal of `SelectionValue` to the four selection forms.
pub trait SelectionValueSeal<K: Clone + PartialEq + 'static> {}
