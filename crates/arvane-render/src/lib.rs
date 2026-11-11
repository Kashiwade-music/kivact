//! Role: Arvane render: rasterises RenderScene into CPU pixmaps and GPU textures.
//!
//! Responsibility: scene rasterization with private vello_cpu and
//! vello_hybrid backends, which share every stage but the last (`draw.rs`).
//! [`CpuRaster`] draws a scene into caller-owned premultiplied RGBA8 pixels
//! on the calling thread, deterministically; `GpuRaster` (feature `gpu`)
//! draws it into a texture view the caller encodes into, on the caller's
//! device. Devices, surfaces, external frames and compositing are not this
//! crate's: they belong to the compositor.
//!
//! Images arrive decoded, as straight-alpha RGBA8 `Pixels`; this crate
//! reads no encoded image. The CPU renderer keeps an image's premultiplied
//! copy while consecutive frames draw it (`images`), so a photo left on
//! screen is not converted again every frame; pixels made from a scene's
//! values (pattern tiles at their device size, gradient tiles) are kept
//! likewise (`derived`).
//!
//! Dependencies: vello_hybrid 0.2 uses the same vello_common 0.2 and
//! glifo 0.3 as vello_cpu 0.2 (no vello_hybrid matches vello_cpu 0.3 yet).
//! vello_common supplies the types vello_cpu and vello_hybrid do not
//! re-export: the filters (blur), texture rectangles and image atlas ids;
//! glifo supplies the glyph settings both take but do not re-export. wgpu
//! is the version vello_hybrid uses, built with Vulkan, DirectX 12 and Metal
//! only.
//!
//! Invariant: renderer types never appear in the public API.
#![forbid(unsafe_code)]

mod convert;
mod cpu;
mod derived;
mod draw;
mod error;
#[cfg(feature = "gpu")]
mod gpu;
mod gradient;
mod images;
mod ink;
mod limits;
mod outline;
mod paint;
mod text;
mod validate;

pub use cpu::CpuRaster;
pub use error::RenderError;
#[cfg(feature = "gpu")]
pub use gpu::{Canvas, GpuRaster, RASTER_FORMAT};
pub use ink::glyph_ink_bounds;
pub use limits::MAX_SCENE_DEPTH;
pub use outline::glyph_outline;
pub use validate::{dimensions, raster_plan};
