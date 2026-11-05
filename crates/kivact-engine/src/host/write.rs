//! Role: Write and read API through which kivact changes the engine's tree in a turn.
//!
//! The engine's write API: how kivact changes the engine's tree during a
//! turn ([`Writer`]), and reads it back ([`Reader`]).
//!
//! The host gives kivact owned cloneable write access at startup. Each
//! mutation borrows the shared model for that operation only. Commit checks
//! focus once, records a private event boundary, and synchronizes changes.
//! The host releases all borrows and completes layout before notifying core.
//! Platform effects, window settings and the imports and drops of shared
//! textures run after the turn; fonts register
//! immediately. Trusted structural operations retain local debug assertions.
//!
//! [`Writer`] and [`Reader`] are concrete types owning the same model and
//! text engine through `Rc`, borrowing them for each call only; there is no
//! backend trait per write or deferred model mutation. The active sibling order
//! changes immediately and is packed at revision and geometry-read boundaries.
//! The effects kivact runs
//! write through them directly (create, the typed setters, insert,
//! move_before, remove, set_scene, focus, blur, scroll, effect, set_window,
//! request_frame, observe_layout).
//!
//! Invariants:
//! - The engine trusts kivact's writes: payloads and setters are typed by
//!   node kind, and only local invariants are checked, by `debug_assert`
//!   (a unique nonzero node id, no cycle, an insert of a detached node, a
//!   `before` among the parent's children). There is no second tree to
//!   validate against and no report of an invalid write; the layout lint
//!   is a module of its own (`check`).
//! - A commit checks the focus, moves the revision on, records the private
//!   boundary of the events (`Model::commit_revision`), then synchronizes
//!   (`Model::sync`).
//! - [`Reader`] lends the typed properties and payload of a node to a
//!   closure ([`Reader::with_node`]) and reads scenes, geometry and input
//!   values: kivact never copies a prop. [`Reader::text_measurer`] gives a
//!   draw callback's `Draw::measure_text` the text engine and the node's
//!   inherited text style (the same measurer type image output measures
//!   with), kept for as long as kivact holds it; the `LeafTextMeasurer` a
//!   measure function is lent lives during one layout pass only.

mod reader;
pub(crate) mod typed;
mod writer;

pub use reader::Reader;
pub use writer::Writer;
