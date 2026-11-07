//! Role: Drawables: view subtrees a canvas draws, their handles and binding.
//!
//! A drawable is created in setup and mounted once (`Node::drawable` or
//! `Viewport::drawable`). The canvas draws it with `Draw::drawable`, and a
//! viewport-hosted drawable is exported to the app through its
//! [`DrawableShare`]. Each drawable's export has a turn-level wake: the
//! share's calls wake the drawable's task, which marks the drawable's
//! notifier; its effect then sends the export request to the engine.
//!
//! Invariants:
//! - A drawable is mounted at one place at a time; the binding names its
//!   root node and is cleared only when that same root is removed.
//! - The export registry (`registry`) holds the links of the drawables that
//!   are alive on this thread; engine events for other ids are stale.
//!
//! The leaves: `handle` (the handle and its binding), `mount` (the root
//! mounted by `Node::drawable` and `Viewport::drawable`), `requests` (what
//! is sent to the engine), `events` (what the engine sends back) and
//! `registry` (the live drawables of this thread).

pub(crate) mod events;
mod handle;
mod mount;
mod registry;
mod requests;

pub use handle::{Drawable, drawable};
