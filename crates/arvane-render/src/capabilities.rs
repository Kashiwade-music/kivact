//! Role: Reports render capabilities selected at build time.

pub const GPU_BUILT: bool = cfg!(feature = "gpu");
