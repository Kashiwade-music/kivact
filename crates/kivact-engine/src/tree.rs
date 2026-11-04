//! Role: Authoritative raw node values, stable slots and structural links.
//!
//! The one ID map and one sibling order live here. Editing order is immediately
//! visible and packed before indexed layout. No layout, paint or shaping state
//! belongs to this structural owner. Upper Model lifecycle code initializes and
//! resets derived columns before publishing a slot for reuse.
//!
//! Removal accepts an upper-owned payload callback without naming its types.
//! Derived columns reset before reuse, and each payload drops after that node's
//! structural cleanup, before its descendants and before Writer attribution.
//! Detaching keeps every slot and its state.

mod child_order;
pub(crate) mod node;
mod node_tree;
mod store;

pub(crate) use child_order::Children;
pub(crate) use node_tree::{NodeTree, states_place};
pub(crate) use store::NodeStore;

mod layout_adapter;
