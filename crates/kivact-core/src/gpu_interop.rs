//! Role: GPU texture sharing with an external render engine (experimental).
//!
//! **Experimental** (feature `gpu-interop`, off by default). An App's own
//! render engine (a 3D viewport, a video decoder, a simulation: its own
//! Direct3D 12 or Vulkan device, not Kivact's) draws frames into a pool of
//! GPU textures, and the window draws them on its GPU without reading them
//! back to the CPU. Windows (Direct3D 12 with Direct3D 12, Vulkan with
//! Vulkan) and Linux (Vulkan); on other platforms the feature builds but
//! [`adapter`] stays `None` and an import fails as unsupported.
//!
//! Two levels:
//!
//! - **[`shared_image`]** does the work for the App. It asks the engine's
//!   thread for a pool ([`ShareRequest`], through the [`ShareLink`] the
//!   engine holds) whenever the window's adapter, the asked size
//!   ([`SharedImage::set_size`]) or the import changes; imports what the
//!   engine provides; carries its frames to [`SharedImage::frame`]; and
//!   falls back to pictures drawn on the CPU ([`ShareMode::Pixels`],
//!   [`SharedImage::pixels`]) while the window cannot import.
//! - **[`import`](fn@import)** imports one pool now, for an App that follows the
//!   adapter and the import's status itself; [`ExternalImage::publisher`]
//!   carries frames from the engine's thread to
//!   [`ExternalImage::latest_frame`].
//!
//! The engine's thread with a [`ShareLink`]:
//!
//! 1. Take the newest request ([`ShareLink::wait_request`]). For
//!    [`ShareMode::Gpu`], make the engine's device on that adapter, with
//!    its graphics API, and a pool of the request's size as
//!    [`SharedTextures`] says; [`ShareLink::provide`] it (`unsafe`: the
//!    engine keeps the contract written there), then
//!    [`ShareLink::wait_ready`].
//! 2. Draw frames and [`ShareLink::publish`] each `(slot, value)`; the UI
//!    thread wakes and the canvas that reads [`SharedImage::frame`]
//!    redraws, as new `Pixels` do.
//! 3. Between frames, [`ShareLink::poll_request`]: a newer request (a new
//!    device, a new size, the window drawing on the CPU) replaces the
//!    pool; drop it and serve the new one. For [`ShareMode::Pixels`],
//!    draw on the CPU and [`ShareLink::publish_pixels`].
//! 4. Stop when [`ShareLink::is_closed`]: the image's owner is gone.
//!
//! An imported pool belongs to an owner (the one [`shared_image`] or
//! [`import`](fn@import) was called in): disposing it drops the image, and the window
//! signals the release fence past every frame. The window's device is its
//! own: an image is drawn only in the App's window, by its GPU; a window
//! that draws on the CPU draws nothing for a frame.
//!
//! ```ignore
//! use std::time::Duration;
//!
//! use kivact::gpu_interop::{self, ShareMode};
//! use kivact::prelude::*;
//! use kivact_ui_creative::viewport_surface;
//!
//! fn viewport() -> impl IntoView {
//!     let image = gpu_interop::shared_image(1280, 720);
//!     let link = image.link();
//!     std::thread::spawn(move || {
//!         let mut engine = None;
//!         while !link.is_closed() {
//!             // A newer request replaces the pool: the old engine goes.
//!             let request = match engine {
//!                 Some(_) => link.poll_request(),
//!                 None => link.wait_request(Duration::from_millis(100)),
//!             };
//!             if let Some(request) = request {
//!                 engine = MyEngine::serve(&link, request);
//!             }
//!             if let Some(engine) = &mut engine {
//!                 engine.draw_and_publish(&link);
//!             }
//!         }
//!     });
//!     // Frames on the GPU; the CPU's pictures while the window cannot
//!     // import.
//!     let (frame, pixels) = (image.clone(), image);
//!     viewport_surface(move || pixels.pixels()).gpu_frame(move || frame.frame())
//! }
//! ```
//!
//! `MyEngine::serve` makes a device and a pool for a [`ShareMode::Gpu`]
//! request, provides it and waits until it is ready; for
//! [`ShareMode::Pixels`] it draws on the CPU.

mod import;
mod share;

#[doc(inline)]
pub use import::{ExternalImage, FramePublisher, ImportStatus, adapter, import};
pub(crate) use import::{State, adapter_changed, imported};
#[doc(inline)]
pub use kivact_style::draw::GpuFrame;
#[doc(inline)]
pub use share::link::{ShareError, ShareLink, ShareMode, ShareRequest};
#[doc(inline)]
pub use share::{ShareStatus, SharedImage, shared_image};

/// An adapter's identity, as the graphics APIs report it.
#[doc(inline)]
pub use kivact_types::AdapterMatch;
/// The adapter of the window's GPU device ([`adapter`]).
#[doc(inline)]
pub use kivact_types::GpuAdapter;
/// A graphics API whose textures can be shared.
#[doc(inline)]
pub use kivact_types::GpuBackend;
/// Which kind of adapter an App prefers (`App::gpu_preference`).
#[doc(inline)]
pub use kivact_types::GpuPower;
/// What an App wishes of the window's GPU device (`App::gpu_preference`).
#[doc(inline)]
pub use kivact_types::GpuPreference;
/// An owned OS handle of a shared texture or fence.
#[doc(inline)]
pub use kivact_types::SharedHandle;
/// The pixel format of shared textures.
#[doc(inline)]
pub use kivact_types::SharedTextureFormat;
/// A pool of textures and its two timeline fences, to [`import`](fn@import); its
/// rustdoc is the engine's contract.
#[doc(inline)]
pub use kivact_types::SharedTextures;
