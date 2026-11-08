//! Role: GPU rasterizer: draws scenes into the textures a caller gives it.
//!
//! Responsibility: one vello_hybrid renderer on the caller's device and
//! queue ([`GpuRaster`]), drawing a scene into a texture view the caller
//! encodes into. Devices, surfaces, external frames and readback belong to
//! the compositor, not here.
//!
//! vello_hybrid is vello_cpu's pipeline with its last stage on the GPU: the
//! CPU flattens paths and makes sparse strips exactly as vello_cpu does
//! (`draw.rs` records the same calls for both), and ordinary render
//! pipelines (no compute shaders) fill the strips' pixels into the target.
//! Its GPU memory follows the scene (strips, a depth texture, glyph atlas
//! pages once text is drawn, a texture per image, layer textures once a
//! layer is used), with no fixed working buffers.
//!
//! Invariants:
//! - GPU resources are serially owned by their rasterizer; a rasterizer
//!   is used on the thread that owns its device.
//! - A target view is [`RASTER_FORMAT`], with render-attachment usage, of
//!   the size the raster is asked for.

mod encode;
mod images;
mod raster;

pub use raster::{Canvas, GpuRaster, RASTER_FORMAT};
