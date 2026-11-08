//! Role: Which optional parts this build of the compositor contains.

/// Whether this build contains the GPU executor (feature `gpu`).
pub const GPU_BUILT: bool = cfg!(feature = "gpu");

/// Whether this build can export drawable frames as shared textures (feature `gpu-interop`).
pub const GPU_INTEROP_BUILT: bool = cfg!(feature = "gpu-interop");
