//! Role: An image an engine shares with the UI, imported again on each new device.
//!
//! The UI side of `kivact::gpu_interop::shared_image`, built on the
//! low-level import (`import::import_in`): it asks the engine for a pool
//! ([`ShareRequest`]) when the adapter, the size or the import changes,
//! imports what the engine provides, and follows the import's status.
//! The engine's side is [`link`].
//!
//! Invariants:
//! - A request is issued at creation, and again when what it was made of
//!   changes: the adapter (`Some` or `None`, its generation), the size
//!   (`set_size`), or the import being `Lost`. Issuing one drops the import of
//!   the one before at once (its child owner is disposed).
//! - Each import has a child owner of the image's owner; its status is
//!   mirrored into the link's phase (which the engine's thread waits on) by
//!   an effect of that child owner, so a dropped import stops being
//!   followed; the image's status is read from the link's phase.
//! - The image's owner going closes the link (its cleanup), after its
//!   child owners dropped the imports.

mod app_side;
pub(crate) mod link;

pub use app_side::{SharedImage, shared_image};
pub use link::ShareStatus;
