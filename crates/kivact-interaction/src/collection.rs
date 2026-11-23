//! Role: Per-collection lookups that item parts read in O(1).
//!
//! A collection with many items (list, combobox, menu, select, tabs, tree,
//! table, palette) builds one [`Index`] per change of its items, one
//! [`Membership`] over its selection and one `selector` over its active
//! key. An item reads its own key through them and follows only what
//! concerns that key (PERF-11).

mod index;
mod membership;

pub(crate) use index::{Index, Movement};
pub(crate) use membership::Membership;
