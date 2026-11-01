//! Role: Images an external render engine shares, imported into the window's GPU device.
//!
//! The implementation of `kivact::gpu_interop` (feature `gpu-interop`):
//! the adapter as a tracked environment value, the imports an App asked
//! for, their status, and the frames a canvas draws.
//!
//! Invariants:
//! - The App names each import with an id of its own (`ExternalImageId`,
//!   counted from 1 per App, never reused); an image belongs to the App's
//!   one window.
//! - An import is one `Writer::import_external`, carried out by the host
//!   after the turn and answered by `HostEvent::ExternalImported`; the
//!   cleanup of the owner it was called in writes `Writer::drop_external`,
//!   whatever handles to it are left, and forgets it (a later answer is
//!   ignored).
//! - The adapter signal lives in the App's [`State`] (a runtime extension
//!   the client installs; the root owner makes the signal before the root
//!   is mounted). It starts `None` and follows
//!   `HostEvent::GpuAdapterChanged`. An import that is `Ready` is `Lost`
//!   once the adapter's generation is not the one it was imported into
//!   (a new device, or none).
//! - Each import has one `FramePublisher` state ([`inbox`]) and a task of
//!   its owner that moves the newest published frame into the tracked
//!   `latest` signal; the owner's cleanup closes the state, so publishing
//!   after it does nothing.

mod image;
pub(crate) mod inbox;
mod registry;

pub(crate) use image::import_in;
pub use image::{ExternalImage, FramePublisher, import};
pub use registry::{ImportStatus, adapter};
pub(crate) use registry::{State, adapter_changed, imported};
