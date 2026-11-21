//! Role: Test-only hooks of the driver: the bundled test font, a GPU probe and a device loss.

use std::sync::Arc;

/// The bundled test font (Tuffy, public domain; its license is
/// `assets/fonts/Tuffy-LICENSE.txt`). It has no CJK glyphs.
static TEST_FONT: &[u8] = include_bytes!("../assets/fonts/Tuffy.ttf");

/// The bundled test font: an isolated text engine given it shapes,
/// lays out and draws text the same on every machine.
pub fn test_font() -> Arc<[u8]> {
    Arc::from(TEST_FONT)
}

/// Why no hardware GPU adapter can be used here, or `None` when one can.
pub fn gpu_unavailable() -> Option<String> {
    arvane_compose::probe_adapter(&arvane_compose::GpuEnv::from_env(
        kivact_types::GpuPreference::default(),
    ))
    .err()
    .map(|error| error.to_string())
}

/// Make the driver's GPU device lost, as a device failure does: the window's
/// exports are reported lost and the device restarts once per session.
pub fn lose_gpu_device(window: &mut crate::Window<'_>) {
    window.lose_gpu_device();
}
