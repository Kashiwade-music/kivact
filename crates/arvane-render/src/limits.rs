//! Role: Passive scene validation and frame budget constants.
/// Maximum simultaneously open clip, transform and layer groups in a scene.
/// Render validation remains authoritative for balanced structure and this bound.
pub const MAX_SCENE_DEPTH: usize = 256;
/// The largest blur, in standard deviations of logical pixels.
pub(crate) const MAX_BLUR: f64 = 512.0;
/// The most dash lengths.
pub(crate) const MAX_DASHES: usize = 64;
/// The longest frame side the renderers draw, in physical pixels.
pub(crate) const MAX_SIDE: f64 = 16_384.0;
/// The most pixels a frame the renderers draw may have (64 MiB of RGBA8).
pub(crate) const MAX_PIXELS: f64 = 16_777_216.0;
