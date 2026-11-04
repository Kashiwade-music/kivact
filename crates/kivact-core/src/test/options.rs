//! Role: Configure test fonts, windows and rendering.

use crate::i18n::{Locale, Messages};
use kivact_style::Theme;
use std::sync::Arc;

/// The fonts text is shaped with ([`RenderOptions::fonts`]).
#[derive(Clone, Debug, Default)]
pub enum TestFonts {
    /// Only the bundled test font (Tuffy, no CJK glyphs): the same layout
    /// and pixels on every machine.
    #[default]
    Bundled,
    /// Only these font files (TTF/OTF bytes): deterministic too.
    Only(Vec<Arc<[u8]>>),
    /// The system's fonts, as a running App sees them.
    System,
}

pub use kivact_engine::testing::MirrorMode;

fn mirror_from_env() -> MirrorMode {
    match std::env::var("KIVACT_TEST_MIRROR").as_deref() {
        Ok("off") => MirrorMode::Off,
        Ok("required") => MirrorMode::Required,
        _ => MirrorMode::Auto,
    }
}

/// Options of [`super::render_with`].
#[derive(Clone, Debug)]
pub struct RenderOptions {
    /// Theme of the app under test.
    pub theme: Theme,
    /// Locale of the app under test, as `App::locale` sets it (default
    /// `None`: the app speaks the host's, see
    /// [`RenderOptions::host_locale`]).
    pub locale: Option<Locale>,
    /// The system locale the test host reports (a BCP 47 tag; default
    /// `None`: the system names none, so an app without a locale speaks
    /// `en-US`). Change it while the app runs with
    /// `Screen::set_host_locale`.
    pub host_locale: Option<String>,
    /// Messages of the app under test (default: none, so every component
    /// shows its English text).
    pub messages: Messages,
    /// Start with the host reporting reduced motion.
    pub reduced_motion: bool,
    /// Start with the host reporting a dark system theme.
    pub dark: bool,
    /// Logical window width (default 800).
    pub width: f64,
    /// Logical window height (default 600).
    pub height: f64,
    /// Physical pixels per logical pixel (default 1; the test's, not the
    /// monitor's).
    pub scale: f64,
    /// The fonts text is shaped with (default: the bundled font only).
    pub fonts: TestFonts,
    /// Real window or headless (default: real when possible).
    pub mirror: MirrorMode,
    /// Draw through the GPU surface and read its pixels back. Defaults to
    /// deterministic CPU rendering. An unavailable GPU fails mounting.
    pub gpu: bool,
    /// Display refresh period frame ticks are paced at (default 60 Hz).
    pub frame_period_ns: u64,
    /// Title of the real window (default: the test's name).
    pub title: Option<String>,
    /// How the window's GPU device is made, as `App::gpu_preference`
    /// (feature `gpu-interop`; default: Arvane's own choice). Use it with
    /// `gpu` to test `kivact::gpu_interop` on a graphics API.
    #[cfg(feature = "gpu-interop")]
    pub gpu_preference: crate::gpu_interop::GpuPreference,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            theme: Theme::default(),
            locale: None,
            host_locale: None,
            messages: Messages::new(),
            reduced_motion: false,
            dark: false,
            width: 800.0,
            height: 600.0,
            scale: 1.0,
            fonts: TestFonts::Bundled,
            mirror: mirror_from_env(),
            gpu: false,
            frame_period_ns: kivact_types::defaults::FRAME_PERIOD_NS,
            title: None,
            #[cfg(feature = "gpu-interop")]
            gpu_preference: Default::default(),
        }
    }
}

impl RenderOptions {
    /// Read mirror policy and request GPU rendering with `KIVACT_TEST_GPU=1`.
    pub fn from_env() -> Self {
        Self {
            gpu: std::env::var("KIVACT_TEST_GPU").as_deref() == Ok("1"),
            ..Self::default()
        }
    }
}
