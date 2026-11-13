//! Role: Options of a test host: fonts, window, scale and clock.

use arvane_virtual::{GpuMode, MirrorMode};
use kivact_types::ThemeHint;
use std::sync::Arc;

/// The bundled default test font: Arvane's test font (Tuffy, public
/// domain), which has no CJK glyphs.
pub fn default_font() -> Arc<[u8]> {
    arvane::test_font()
}

/// The fonts text is shaped with.
#[derive(Clone, Debug, Default)]
pub enum TestFonts {
    /// Only the bundled default font ([`default_font`]): deterministic.
    #[default]
    Bundled,
    /// Only these fonts: deterministic.
    Only(Vec<Arc<[u8]>>),
    /// The system's fonts, as a running App sees them.
    System,
}

/// Options of [`ArvaneTestHost::new`].
#[derive(Clone, Debug)]
pub struct TestHostOptions {
    /// Read-only state restored at launch.
    pub restored_state: std::collections::BTreeMap<String, Vec<u8>>,
    /// Logical content width.
    pub width: f64,
    /// Logical content height.
    pub height: f64,
    /// Physical pixels per logical pixel (the test's, not the monitor's).
    pub scale: f64,
    /// The fonts text is shaped with.
    pub fonts: TestFonts,
    /// Window mirroring.
    pub mirror: MirrorMode,
    /// Title of the real window.
    pub title: String,
    /// The test platform opens no window beyond the first.
    pub single_window: bool,
    /// The test platform opens windows whose content is one native child
    /// window (`WindowContent::Foreign`).
    pub foreign_windows: bool,
    /// The display refresh period frame ticks are paced at (ns).
    pub frame_period_ns: u64,
    /// The theme hint reported at start.
    pub theme: ThemeHint,
    /// The reduced-motion setting reported at start.
    pub reduced_motion: bool,
    /// The system locale (a BCP 47 tag) reported at start; `None`: the
    /// system names none.
    pub locale: Option<String>,
    /// Load images on the calling thread when they are requested (a
    /// deterministic layout); `false` uses the asset store's worker thread
    /// like a running App (results arrive in real time and wake the host).
    pub inline_assets: bool,
    /// The window can draw on the GPU, through a GPU surface of the real
    /// window (requires a native mirror). The device starts as
    /// `gpu_preference.start` says: with the window (the host then waits
    /// until it is ready, and fails to start when it cannot) or when the
    /// window needs it. `Off`: the CPU only.
    pub gpu: GpuMode,
    /// How the window's GPU device is made, as a native window's
    /// (`HostOptions::gpu_preference`).
    pub gpu_preference: kivact_types::GpuPreference,
}

impl Default for TestHostOptions {
    fn default() -> Self {
        Self {
            restored_state: Default::default(),
            width: 400.0,
            height: 300.0,
            scale: 1.0,
            fonts: TestFonts::Bundled,
            mirror: MirrorMode::Auto,
            title: "kivact test".into(),
            single_window: false,
            foreign_windows: false,
            frame_period_ns: kivact_types::defaults::FRAME_PERIOD_NS,
            theme: ThemeHint::Light,
            reduced_motion: false,
            locale: None,
            inline_assets: true,
            gpu: GpuMode::Off,
            gpu_preference: Default::default(),
        }
    }
}
