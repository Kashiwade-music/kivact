//! Role: Arvane render: rasterises `kivact-types` draw commands.
//!
//! Responsibility: scene rasterization with private vello_cpu and
//! vello_hybrid backends, which share every stage but the last (`draw.rs`).
//! The CPU path is deterministic and backs the tests' frames; the GPU path
//! (feature `gpu`) presents to window surfaces.
//! Images arrive decoded, as straight-alpha RGBA8
//! `Pixels`; this crate reads no encoded image. The
//! CPU renderer keeps an image's premultiplied copy while consecutive
//! frames draw it (`images`), so a photo left on screen is not converted
//! again every frame; pixels made from a scene's values (images through a
//! color matrix, pattern tiles at their device size) are kept likewise
//! (`derived`). What vello_hybrid lacks (mask layers, destructive
//! blending, color matrix filters, reading pixels back) is drawn in forms
//! both rasterizers have (`draw/effects.rs`), and a scene with backdrops
//! is indexed by where its commands draw (`backdrop`). With the feature
//! `gpu-interop`, the GPU path also draws the textures an application's
//! render engine shares (`RenderCmd::External`, `gpu/external.rs`),
//! imported through `kivact-gpu-share`, which holds the `unsafe` below
//! wgpu that needs; the CPU path never draws them.
//!
//! Dependencies: vello_hybrid 0.2 uses the same vello_common 0.2 and
//! glifo 0.3 as vello_cpu 0.2 (no vello_hybrid matches vello_cpu 0.3 yet).
//! vello_common supplies the types vello_cpu and vello_hybrid do not
//! re-export: the filters (blur, drop shadow), texture rectangles and image
//! atlas ids; glifo supplies the glyph settings both take but do not
//! re-export. wgpu is the version vello_hybrid uses, built with Vulkan,
//! DirectX 12 and Metal only; raw-window-handle is how a GPU surface takes
//! its window; bytemuck views softbuffer's buffer as vello_cpu's bytes.
//!
//! Invariant: renderer types never appear in the public API.

//! Successful font-face metadata is retained per renderer with weak byte
//! identity, face index and a 256-entry LRU bound. Every scene still checks
//! glyph ids, variation values, colors and composed placement coordinates.
//! `release_caches` drops only rebuildable validation metadata, preserving
//! imports, leases and drawing resources already in use.
#![forbid(unsafe_code)]

mod backdrop;
mod error;
mod gradient;
pub use error::RenderError;
mod convert;
mod cpu;
mod derived;
mod draw;
#[cfg(feature = "gpu")]
pub(crate) mod gpu;
#[cfg(not(feature = "gpu"))]
mod gpu_unbuilt;
mod images;
mod ink;
mod limits;
mod paint;
mod text;
mod validate;

pub use cpu::CpuRenderer;
#[cfg(feature = "gpu")]
pub use gpu::state::{GpuContext, GpuEnv, GpuSurface, Presentation};
#[cfg(not(feature = "gpu"))]
pub use gpu_unbuilt::{GpuContext, GpuEnv, GpuSurface, Presentation};

pub const GPU_BUILT: bool = cfg!(feature = "gpu");

pub use ink::glyph_ink_bounds;
pub use validate::{dimensions, raster_plan};

pub use limits::MAX_SCENE_DEPTH;
