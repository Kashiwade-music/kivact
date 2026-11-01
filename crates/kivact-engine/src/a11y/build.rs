//! Role: Construct dirty accessibility payloads using the full builder's policy.
//!
//! Identity/parent order, clipping and raw relation dependencies are enumerated
//! without constructing payloads. Subtree dirt and reverse edges expand once;
//! only selected nodes call build_node. The independent debug reference calls
//! the same policy for every node with disposable metadata and EditLines.
//!
//! Invariants:
//! - Bounds are the untransformed content-space box: the node's layout box
//!   with the scrolls of its ancestors undone. A scroll container carries
//!   its offset, a transformed node the transform about its box. A layer
//!   (an overlay, a fixed box) is drawn at its window origin whatever its
//!   ancestors' transforms, so under a transformed ancestor it carries the
//!   transform that undoes theirs.
//! - A relation (`described_by`, `controls`, `labelled_by`) keeps, in the
//!   order kivact wrote them, only the targets the published tree has: a
//!   node (or a scene region's node) that exists, is attached to the root
//!   and is exposed (has a role). A hidden target stays, as ARIA allows.
//!   Old and current raw reverse edges invalidate sources when targets change,
//!   including relations temporarily filtered from the accepted payload.
//! - An active descendant is kept only while it is shown, has a role (a
//!   region: neither it nor its node disabled, and the region not hidden)
//!   and lies in the node's subtree or in the subtree of a node it
//!   `controls` (the node itself included: the ARIA combobox pattern of a
//!   text field and its sibling list). The controls are checked first, so
//!   a dropped relation does not admit one.
//! - Level, set size and position follow `properties` (one-based, `0`
//!   unset, no position past the set's size).
//! - A text input publishes no value (that would copy and compare its whole
//!   text): its text goes as the text runs of the lines around its caret
//!   and those shown (`editing_semantics`). A password publishes neither its
//!   value nor a text snapshot. Assistive technology can set an input's
//!   value (`SetValue`, as typing), but a request to select a range is not
//!   supported: no such action exists.

mod dirty;
mod identity;
mod local_text;
mod node;
mod relations;
