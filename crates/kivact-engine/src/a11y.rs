//! Role: Accessibility tree built from the model's semantics and handed to the window.
//!
//! The accessibility tree handed to the window: the model's semantics
//! policy (`model::semantics`: the role, the accessible name, the actions
//! a node and its scene regions offer) mapped onto Arvane's semantics,
//! which the window hands to AccessKit; and assistive technology's
//! requests mapped back onto the model.
//!
//! Invariants:
//! - What a node is exposed with is the model's
//!   (`Model::exposed`: role, name, actions): nothing offered while it
//!   is disabled (the node itself, never its descendants), not shown, or
//!   outside the shown modal; scene regions likewise. A tab stop in scope
//!   also offers `Focus`, which AccessKit needs to move the focus.
//! - A node without a role is not exposed: it stays in the tree only as a
//!   bare generic container for its children (AccessKit's platform
//!   adapters leave such a node out and lift its children to the parent),
//!   with no name, state, test id, actions, focus or scene regions, and
//!   assistive technology's requests on it are ignored.
//! - Bounds are untransformed layout boxes in content space (the scrolls
//!   above a node undone); a scroll container carries its offset and a
//!   transformed node the transform around its box, the one it is drawn
//!   and hit with (G-07), so the platform maps it as it is drawn (`build`).
//! - Sync records one `Dirt` level (clean, local text, metadata with its
//!   own-node, subtree and removed sets, or full). Publication enumerates
//!   identity/order metadata, constructs only dirty payloads, and compares
//!   complete fingerprints; the build returns the metadata it was built with,
//!   which only window acceptance commits (with the hashes). No accepted
//!   payload tree is retained in the engine. A rejected update requests a
//!   complete resynchronization (`Dirt::Full`).
//! - Publication happens after the client's turns, or, when the native
//!   window has a frame to draw, after that frame
//!   (`Window::defers_accessibility`).
//! - DrawList regions are flat synthetic leaves of their node, with native ids
//!   counted down from `u64::MAX` that stay stable while the region exists;
//!   a region with a non-finite or negative rect is left out alone.
//! - Rows of a clipping list-like container (virtual list overscan) lying
//!   wholly outside it beyond one neighbour are hidden, unless they hold
//!   the focus (AccessKit's `clips_children` filter).
//! - AccessKit has one default action: `Activate`, `Toggle` and `Select`
//!   resolve to the first one offered and activate the target.
mod actions;
mod build;
mod hash;
mod publish;
pub(crate) mod state;
mod text;
